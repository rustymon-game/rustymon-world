use std::collections::HashMap;
use std::path::Path;

use aho_corasick::AhoCorasick;

use crate::features::config::{Ast, Branch, ConfigParser};
use crate::features::simple::eval_expr;
use crate::features::{FeatureParser, Tags};

#[derive(Default, Debug)]
pub struct Tokens<'a> {
    map: HashMap<&'a str, usize>,
    len: usize,
}

impl<'a> Tokens<'a> {
    pub fn get_or_create(&mut self, token: &'a str) -> usize {
        if let Some(id) = self.map.get(token) {
            *id
        } else {
            self.map.insert(token, self.len);
            self.len += 1;
            self.len - 1
        }
    }
}

impl<'a> From<Tokens<'a>> for AhoCorasick {
    fn from(tokens: Tokens) -> Self {
        Self::new(tokens.map.into_keys())
    }
}

pub struct ACParser {
    pub ast: Ast<usize>,
    pub tokenizer: AhoCorasick,
}

impl FeatureParser for ACParser {
    type AreaFeature = usize;
    type NodeFeature = usize;
    type WayFeature = usize;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::AreaFeature> {
        self.parse_tags(&self.ast.areas, area)
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::NodeFeature> {
        self.parse_tags(&self.ast.nodes, node)
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::WayFeature> {
        self.parse_tags(&self.ast.ways, way)
    }
}

impl ACParser {
    pub fn from_file(path: impl AsRef<Path>) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut tokens = Tokens::default();
        let parser = ConfigParser::new(|string| tokens.get_or_create(string));
        let ast = parser.parse_file(&content).ok()?;
        let tokenizer = AhoCorasick::from(tokens);
        Some(Self { tokenizer, ast })
    }

    fn parse_tags<'t>(&self, statements: &[Branch<usize>], tags: impl Tags<'t>) -> Option<usize> {
        let str_to_usize = |tag: &str| -> Option<usize> {
            let result = self.tokenizer.find(tag)?;
            if result.start() == 0 && result.end() == tag.len() {
                None
            } else {
                Some(result.pattern())
            }
        };
        let tags = tags
            .into_iter()
            .filter_map(|(key, value)| {
                str_to_usize(key).map(|key| (key, str_to_usize(value).unwrap_or(usize::MAX)))
            })
            .collect();
        for statement in statements {
            if eval_expr(&statement.expr, &tags) {
                return Some(statement.id);
            }
        }
        None
    }
}
