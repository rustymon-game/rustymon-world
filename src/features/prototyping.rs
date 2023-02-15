//! This feature and parser is intended to be used while prototyping to visualize unoptimised spawn rules.

use linear_map::LinearMap;
use yada::builder::DoubleArrayBuilder;
use yada::DoubleArray;

use crate::features::{FeatureParser, Tags};

pub struct Parser {
    keys: DoubleArray<Vec<u8>>,
    values: Vec<DoubleArray<Vec<u8>>>,
}

impl Parser {
    pub fn from_file(file: &str) -> Option<Self> {
        let config: LinearMap<String, Vec<String>> = serde_json::from_str(file).ok()?;

        let mut keys: Vec<_> = config
            .keys()
            .enumerate()
            .map(|(i, k)| (k.as_str(), i as u32))
            .collect();
        keys.sort_by_key(|(k, _)| *k);

        let mut parser = Self {
            keys: DoubleArray::new(DoubleArrayBuilder::build(&keys)?),
            values: Vec::with_capacity(config.values().len()),
        };

        for values in config.values() {
            let mut values: Vec<_> = values
                .iter()
                .enumerate()
                .map(|(i, v)| (v.as_str(), i as u32))
                .collect();
            values.sort_by_key(|(v, _)| *v);
            let values = DoubleArrayBuilder::build(&values)?;
            parser.values.push(DoubleArray::new(values));
        }

        Some(parser)
    }

    fn parse<'t>(&self, tags: impl Tags<'t>) -> Option<Feature> {
        let mut feature = Vec::new();
        for (key, value) in tags {
            if let Some(key) = self.keys.exact_match_search(key) {
                if let Some(value) = self.values[key as usize].exact_match_search(value) {
                    feature.push([key, value]);
                }
            }
        }
        (!feature.is_empty()).then_some(feature)
    }
}

type Feature = Vec<[u32; 2]>;

impl FeatureParser for Parser {
    type Feature = Feature;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse(area)
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse(node)
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::Feature> {
        self.parse(way)
    }
}
