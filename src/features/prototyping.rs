//! This feature and parser is intended to be used while prototyping to visualize unoptimised spawn rules.

use crate::features::{FeatureParser, Tags};
use linear_map::LinearMap;
use yada::builder::DoubleArrayBuilder;
use yada::DoubleArray;

struct Parser {
    keys: DoubleArray<Vec<u8>>,
    values: Vec<DoubleArray<Vec<u8>>>,
}

impl Parser {
    pub fn from_file(file: &str) -> Option<Self> {
        let config: LinearMap<String, Vec<String>> = serde_json::from_str(file).ok()?;

        let keys = DoubleArray::new(DoubleArrayBuilder::build(
            &config
                .keys()
                .enumerate()
                .map(|(i, k)| (k, i as u32))
                .collect::<Vec<(&str, u32)>>(),
        )?);

        let mut values = config
            .values()
            .map(|values| {
                DoubleArray::new(DoubleArrayBuilder::build(
                    &values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (v, i as u32))
                        .collect::<Vec<(&str, u32)>>(),
                ))
            })
            .collect();

        Some(Self { keys, values })
    }

    fn parse<'t>(&self, tags: impl Tags<'t>) -> Option<Feature> {
        let mut feature = Vec::new();
        for (key, value) in tags {
            if let Some(key) = self.keys.exact_match_search(key) {
                if let Some(value) = self.values[key as usize].exact_match_search(value) {
                    feature.push((key, value));
                }
            }
        }
        !feature.is_empty().then_some(feature)
    }
}

type Feature = Vec<(u32, u32)>;

impl FeatureParser for Parser {
    type AreaFeature = Feature;
    type NodeFeature = Feature;
    type WayFeature = Feature;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::AreaFeature> {
        self.parse(area)
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::NodeFeature> {
        self.parse(node)
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::WayFeature> {
        self.parse(way)
    }
}
