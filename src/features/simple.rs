use libosmium::tag_list::TagList;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::features::FeatureParser;

/// Simple parser for visual types
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SimpleVisual {
    pub areas: Vec<Branch<String>>,
    pub nodes: Vec<Branch<String>>,
    pub ways: Vec<Branch<String>>,
}

pub type Branch<T> = (HashMap<T, Pattern<T>>, usize);

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(untagged)]
pub enum Pattern<T: Eq + Hash> {
    #[default]
    Any,
    Single(T),
    Set(HashSet<T>),
}

impl FeatureParser for SimpleVisual {
    type AreaFeature = usize;
    type NodeFeature = usize;
    type WayFeature = usize;

    fn area(&self, tags: &TagList) -> Option<Self::AreaFeature> {
        parse_tags(tags, &self.areas)
    }

    fn node(&self, tags: &TagList) -> Option<Self::NodeFeature> {
        parse_tags(tags, &self.nodes)
    }

    fn way(&self, tags: &TagList) -> Option<Self::WayFeature> {
        parse_tags(tags, &self.ways)
    }
}

/// Parse tags using the simple "match-ish" format
///
/// ## Generics:
/// This method is generic over the **K**ey and **V**alue used to form the tags
/// as well as their **O**wned and **B**orrowed versions.
pub(crate) fn parse_tags<'a, Iter, KO, KB, VO, VB>(
    tags: Iter,
    lookup: &[(HashMap<KO, Pattern<VO>>, usize)],
) -> Option<usize>
where
    Iter: IntoIterator<Item = (&'a KB, &'a VB)>,
    KO: Eq + Hash + Borrow<KB>,
    KB: Eq + Hash + ?Sized + 'a,
    VO: Eq + Hash + Borrow<VB>,
    VB: Eq + Hash + ?Sized + 'a,
{
    let tags: HashMap<&'a KB, &'a VB> = HashMap::from_iter(tags);

    for (map, result) in lookup {
        let mut matches = true;
        for (exp_key, exp_value) in map {
            if let Some(&tag_value) = tags.get(exp_key.borrow()) {
                match exp_value {
                    Pattern::Any => continue,
                    Pattern::Single(exp_value) if exp_value.borrow() == tag_value => continue,
                    Pattern::Set(exp_values) if exp_values.contains(tag_value) => continue,
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
