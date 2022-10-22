//! Translate OSM features (i.e. tags) into rustymon features
//!
//! For example turn a real world shop into a virtual world one

use crate::formats::{AreaVisualType, NodeVisualType, WayVisualType};
use libosmium::tag_list::TagList;

/// A parser for converting OSM tags into rustymon visual types
pub trait VisualParser {
    fn area(&self, tags: &TagList) -> AreaVisualType;
    fn node(&self, tags: &TagList) -> NodeVisualType;
    fn way(&self, tags: &TagList) -> WayVisualType;
}

/// Decide whether or not an area, node or way should be processed, based on its tags.
pub trait Filter {
    /// Should this area be processed?
    fn area(&self, area: &TagList) -> bool;

    /// Should this node be processed?
    fn node(&self, node: &TagList) -> bool;

    /// Should this way be processed?
    fn way(&self, way: &TagList) -> bool;
}

/// Filter accepting anything which contains at least one tag
pub struct NonEmpty;
impl Filter for NonEmpty {
    #[inline(always)]
    fn area(&self, area: &TagList) -> bool {
        !area.is_empty()
    }

    #[inline(always)]
    fn node(&self, node: &TagList) -> bool {
        !node.is_empty()
    }

    #[inline(always)]
    fn way(&self, way: &TagList) -> bool {
        !way.is_empty()
    }
}

/// Default filter accepting everything
impl Filter for () {
    #[inline(always)]
    fn area(&self, _area: &TagList) -> bool {
        true
    }

    #[inline(always)]
    fn node(&self, _node: &TagList) -> bool {
        true
    }

    #[inline(always)]
    fn way(&self, _way: &TagList) -> bool {
        true
    }
}
