use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

use crate::features::{simple, FeatureParser, Tags};

/// The config format read from disk
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BorrowedConfig<'a> {
    #[serde(borrow)]
    pub areas: Vec<simple::Branch<&'a str>>,

    #[serde(borrow)]
    pub nodes: Vec<simple::Branch<&'a str>>,

    #[serde(borrow)]
    pub ways: Vec<simple::Branch<&'a str>>,
}

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
    pub areas: Vec<simple::Branch<usize>>,
    pub nodes: Vec<simple::Branch<usize>>,
    pub ways: Vec<simple::Branch<usize>>,

    pub tokenizer: AhoCorasick,
}

impl FeatureParser for ACParser {
    type AreaFeature = usize;
    type NodeFeature = usize;
    type WayFeature = usize;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::AreaFeature> {
        let area = self.tokenize(area);
        simple::parse_tags(
            area.iter()
                .map(|(k, v)| (k, v.as_ref().unwrap_or(&usize::MAX))),
            &self.areas,
        )
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::NodeFeature> {
        let node = self.tokenize(node);
        simple::parse_tags(
            node.iter()
                .map(|(k, v)| (k, v.as_ref().unwrap_or(&usize::MAX))),
            &self.nodes,
        )
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::WayFeature> {
        let way = self.tokenize(way);
        simple::parse_tags(
            way.iter()
                .map(|(k, v)| (k, v.as_ref().unwrap_or(&usize::MAX))),
            &self.ways,
        )
    }
}

impl ACParser {
    fn find(&self, tag: &str) -> Option<usize> {
        let result = self.tokenizer.find(tag)?;
        if result.start() == 0 && result.end() == tag.len() {
            None
        } else {
            Some(result.pattern())
        }
    }

    fn tokenize<'t>(&self, tags: impl Tags<'t>) -> Vec<(usize, Option<usize>)> {
        let mut tokens = Vec::new();
        for (key, value) in tags {
            let Some(key) = self.find(key) else {
                continue;
            };
            let value = self.find(value);
            tokens.push((key, value));
        }
        tokens
    }

    pub fn from_file(path: impl AsRef<Path>) -> Option<Self> {
        let file = File::open(path).ok()?;
        let string = std::io::read_to_string(file).ok()?;
        let config: BorrowedConfig = serde_json::from_str(&string).ok()?;

        let mut tokens = Tokens::default();
        let mut parse = {
            let tokens = &mut tokens;
            move |branch: (HashMap<_, _>, usize)| -> (HashMap<_, _>, usize) {
                (
                    branch
                        .0
                        .into_iter()
                        .map(|(key, pattern)| {
                            (
                                tokens.get_or_create(key),
                                match pattern {
                                    simple::Pattern::Any => simple::Pattern::Any,
                                    simple::Pattern::Single(value) => {
                                        simple::Pattern::Single(tokens.get_or_create(value))
                                    }
                                    simple::Pattern::Set(values) => simple::Pattern::Set(
                                        values
                                            .into_iter()
                                            .map(|value| tokens.get_or_create(value))
                                            .collect(),
                                    ),
                                },
                            )
                        })
                        .collect(),
                    branch.1,
                )
            }
        };

        Some(Self {
            areas: config.areas.into_iter().map(&mut parse).collect(),
            nodes: config.nodes.into_iter().map(&mut parse).collect(),
            ways: config.ways.into_iter().map(&mut parse).collect(),
            tokenizer: tokens.into(),
        })
    }
}
