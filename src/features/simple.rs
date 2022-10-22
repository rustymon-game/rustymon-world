use crate::features::VisualParser;
use crate::formats::{AreaVisualType, NodeVisualType, WayVisualType};
use libosmium::tag_list::TagList;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(untagged)]
pub enum StringPattern {
    #[default]
    Any,
    Single(String),
    Set(HashSet<String>),
}

/// Simple parser for visual types
#[derive(Deserialize, Default, Debug)]
pub struct SimpleVisual {
    pub areas: Vec<(HashMap<String, StringPattern>, AreaVisualType)>,
    pub nodes: Vec<(HashMap<String, StringPattern>, NodeVisualType)>,
    pub ways: Vec<(HashMap<String, StringPattern>, WayVisualType)>,
}

impl VisualParser for SimpleVisual {
    fn area(&self, tags: &TagList) -> AreaVisualType {
        get_type(tags, &self.areas).unwrap_or(AreaVisualType::None)
    }

    fn node(&self, tags: &TagList) -> NodeVisualType {
        get_type(tags, &self.nodes).unwrap_or(NodeVisualType::None)
    }

    fn way(&self, tags: &TagList) -> WayVisualType {
        get_type(tags, &self.ways).unwrap_or(WayVisualType::None)
    }
}

fn get_type<T: Copy>(tags: &TagList, lookup: &[(HashMap<String, StringPattern>, T)]) -> Option<T> {
    let tags: HashMap<String, String> = HashMap::from_iter(
        tags.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    );

    for (map, result) in lookup {
        let mut matches = true;
        for (exp_key, exp_value) in map {
            if let Some(tag_value) = tags.get(exp_key) {
                match exp_value {
                    StringPattern::Any => continue,
                    StringPattern::Single(exp_value) if exp_value == tag_value => continue,
                    StringPattern::Set(exp_values) if exp_values.contains(tag_value) => continue,
                    _ => {
                        matches = false;
                        break;
                    }
                }
            } else {
                matches = false;
                break;
            }
        }

        if matches {
            return Some(*result);
        } else {
            continue;
        }
    }

    None
}
