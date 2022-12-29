use std::collections::HashMap;

use yada::builder::DoubleArrayBuilder;
use yada::DoubleArray;

use crate::features::config::{Ast, Branch, ConfigParser};
use crate::features::simple::eval_expr;
use crate::features::{FeatureParser, Tags};

#[derive(Default)]
pub struct Tokens {
    pub strings: HashMap<String, u32>,
}

impl Tokens {
    pub fn get_or_insert(&mut self, string: &str) -> u32 {
        if let Some(index) = self.strings.get(string) {
            *index
        } else {
            let index = self.strings.len() as u32;
            assert_eq!(
                index >> 31 & 1,
                0,
                "The most significant bit is reserved for yada!\n"
            );
            self.strings.insert(string.to_string(), index);
            index
        }
    }
}

impl Tokens {
    pub fn finish(self) -> Result<DoubleArray<Vec<u8>>, &'static str> {
        fn get_first<'t, 's>(tuple: &'t (&'s str, u32)) -> &'s str {
            tuple.0
        }

        let mut keyset: Vec<_> = self
            .strings
            .iter()
            .map(|(string, index)| (string.as_str(), *index))
            .collect();
        keyset.sort_by_key(get_first);

        Ok(DoubleArray::new(
            DoubleArrayBuilder::build(&keyset).ok_or("Couldn't build trie")?,
        ))
    }
}

pub struct YadaParser {
    pub ast: Ast<u32>,
    pub tokenizer: DoubleArray<Vec<u8>>,
}

impl FeatureParser for YadaParser {
    type Feature = usize;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse_tags(&self.ast.areas, area)
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse_tags(&self.ast.nodes, node)
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse_tags(&self.ast.ways, way)
    }
}

impl YadaParser {
    pub fn from_file(file: &str) -> Option<Self> {
        let mut tokens = Tokens::default();
        let parser = ConfigParser::new(|string| tokens.get_or_insert(string));
        let ast = parser.parse_file(&file).ok()?;
        let tokenizer = tokens.finish().ok()?;
        Some(Self { tokenizer, ast })
    }

    fn parse_tags<'t>(&self, statements: &[Branch<u32>], tags: impl Tags<'t>) -> Option<usize> {
        let get = |tag| self.tokenizer.exact_match_search(tag);
        let tags = tags
            .into_iter()
            .filter_map(|(key, value)| get(key).map(|key| (key, get(value).unwrap_or(u32::MAX))))
            .collect();
        for statement in statements {
            if eval_expr(&statement.expr, &tags) {
                return Some(statement.id);
            }
        }
        None
    }
}
