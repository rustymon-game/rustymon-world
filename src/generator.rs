use libosmium::handler::Handler;
use libosmium::node_ref_list::NodeRefList;
use libosmium::{Area, Node, Way, PRECISION};
use nalgebra::Vector2;

use crate::features::FeatureParser;
use crate::formats::Tile;
use crate::geometry::bbox::GenericBox;
use crate::geometry::grid::Grid;
use crate::geometry::polygon::combine_rings;
use crate::geometry::{BBox, Point};
use crate::projection::Projection;

#[derive(Clone)]
pub struct WorldGenerator<P: Projection, V: FeatureParser> {
    pub int_box: GenericBox<i32>,
    pub projection: P,

    // Buffer to copy rings into before combining them.
    pub rings: Vec<Vec<Point>>,

    // Grid
    pub grid: Grid,
    pub tiles: Vec<Tile<V::Feature>>,

    // Current visual types
    pub visual_parser: V,
    pub area_type: V::Feature,
    pub node_type: V::Feature,
    pub way_type: V::Feature,
}

impl<P: Projection, V: FeatureParser> WorldGenerator<P, V> {
    pub fn new(
        center: Point,
        (num_cols, num_rows): (usize, usize),
        zoom: u8,
        visual_parser: V,
        projection: P,
    ) -> Self
    where
        V::Feature: Default,
    {
        // A tiles size in the map's coordinates
        let step_size = 1.0 / (1 << zoom) as f64;
        let step_size = Vector2::new(step_size, step_size);

        // The "min" corner of the center tile.
        let mut center = projection.project_nalgebra(center);
        center.x -= center.x % step_size.x;
        center.x -= center.y % step_size.y;

        // The "min" corner of the entire grid
        let min = Vector2::new(
            center.x - num_cols as f64 * step_size.x / 2.0,
            center.y - num_rows as f64 * step_size.y / 2.0,
        );

        let mut tiles = Vec::with_capacity(num_cols * num_rows);
        for y in 0..num_rows {
            for x in 0..num_cols {
                let min = Vector2::new(
                    min.x + x as f64 * step_size.x,
                    min.y + y as f64 * step_size.y,
                );
                tiles.push(Tile::new(BBox {
                    min,
                    max: min + step_size,
                }));
            }
        }

        let bbox = BBox {
            min,
            max: Vector2::new(
                min.x + num_cols as f64 * step_size.x,
                min.y + num_rows as f64 * step_size.y,
            ),
        };

        WorldGenerator {
            int_box: GenericBox {
                min: bbox.min.map(|f| (f * PRECISION as f64).floor() as i32),
                max: bbox.max.map(|f| (f * PRECISION as f64).ceil() as i32),
            },
            projection,

            rings: Vec::new(),

            grid: Grid::new(min, Vector2::new(num_cols, num_rows), step_size),
            tiles,

            visual_parser,
            area_type: Default::default(), // Only every read
            node_type: Default::default(), // directly after
            way_type: Default::default(),  // assignment.
        }
    }

    pub fn into_tiles(self) -> Vec<Tile<V::Feature>> {
        std::mem::forget(self.area_type);
        std::mem::forget(self.node_type);
        std::mem::forget(self.way_type);
        self.tiles
    }

    fn iter_nodes(projection: P, nodes: &NodeRefList) -> impl Iterator<Item = Point> + '_ {
        nodes
            .iter()
            .filter_map(move |node| projection.project(node))
    }
}

impl<P: Projection, V: FeatureParser> Handler for WorldGenerator<P, V>
where
    V::Feature: Clone,
{
    fn area(&mut self, area: &Area) {
        if area.tags().is_empty() {
            return;
        }
        if let Some(feature) = self.visual_parser.area(area.tags()) {
            self.area_type = feature;
        } else {
            return;
        }

        for ring in area.outer_rings() {
            let mut polygon: Vec<Point> = Self::iter_nodes(self.projection, ring).collect();

            // Collect the inner rings into reused vectors
            let mut num_rings = 0;
            for inner_ring in area.inner_rings(ring) {
                // Reuse old Vec or push new one
                if num_rings < self.rings.len() {
                    self.rings[num_rings].clear();
                    let inner_ring = Self::iter_nodes(self.projection, inner_ring);
                    self.rings[num_rings].extend(inner_ring);
                } else {
                    self.rings
                        .push(Self::iter_nodes(self.projection, inner_ring).collect());
                }

                // Only count non-empty rings
                if !self.rings[num_rings].is_empty() {
                    num_rings += 1;
                }
            }
            // Add the inner rings to the outer ring before clipping
            if num_rings > 0 {
                combine_rings(&mut polygon, &mut self.rings[0..num_rings]);
            }

            self.grid.clip_polygon(polygon, |index, polygon| {
                if let Some(tile) = self.tiles.get_mut(index) {
                    if !polygon.is_empty() {
                        tile.add_area(polygon, self.area_type.clone());
                    }
                }
            });
        }
    }

    fn node(&mut self, node: &Node) {
        if node.tags().is_empty() {
            return;
        }
        if let Some(feature) = self.visual_parser.node(node.tags()) {
            self.node_type = feature;
        } else {
            return;
        }

        if let Some(point) = self.projection.project(node) {
            self.grid.clip_point(point, |index, point| {
                if let Some(tile) = self.tiles.get_mut(index) {
                    tile.add_node(point, self.node_type.clone());
                }
            });
        }
    }

    fn way(&mut self, way: &Way) {
        if way.tags().is_empty() {
            return;
        }
        if let Some(feature) = self.visual_parser.way(way.tags()) {
            self.way_type = feature;
        } else {
            return;
        }

        let nodes = way.nodes();

        // Skip closed ways (only checking nodes' ids)
        match (nodes.first(), nodes.last()) {
            (Some(first), Some(last)) => {
                if first.id == last.id {
                    return;
                }
            }
            _ => return,
        }

        self.grid
            .clip_path(Self::iter_nodes(self.projection, nodes), |index, path| {
                if let Some(tile) = self.tiles.get_mut(index) {
                    tile.add_way(path, self.way_type.clone());
                }
            });
    }
}
