//! Translate OSM features (i.e. tags) into rustymon features
//!
//! For example turn a real world shop into a virtual world one

pub mod config;
pub mod pest_ext;
pub mod simple;
#[cfg(feature = "yada")]
pub mod yada;

/// Trait alias for a `IntoIterator` over pairs of `&'t str`
pub trait Tags<'t>: IntoIterator<Item = (&'t str, &'t str)> {}
impl<'t, T: IntoIterator<Item = (&'t str, &'t str)>> Tags<'t> for T {}

pub trait FeatureParser {
    type AreaFeature: 'static;
    type NodeFeature: 'static;
    type WayFeature: 'static;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::AreaFeature>;
    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::NodeFeature>;
    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::WayFeature>;
}
