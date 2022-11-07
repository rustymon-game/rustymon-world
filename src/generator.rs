use crate::features::VisualParser;
use crate::formats;
use crate::formats::{AreaVisualType, Constructable, NodeVisualType, WayVisualType};
use crate::geometry::bbox::GenericBox;
use crate::geometry::grid::{Grid, Index};
use crate::geometry::polygon::combine_rings;
use crate::geometry::{BBox, Point};
use crate::projection::Projection;
use libosmium::handler::Handler;
use libosmium::node_ref_list::NodeRefList;
use libosmium::{Area, Node, Way, PRECISION};
use nalgebra::Vector2;

pub struct WorldGenerator<P: Projection, V: VisualParser> {
    pub int_box: GenericBox<i32>,
    pub projection: P,

    // Buffer to copy rings into before combining them.
    pub rings: Vec<Vec<Point>>,

    // Grid
    pub bbox: BBox,
    pub step: Vector2<f64>,
    pub size: Vector2<isize>,
    pub tiles: Vec<formats::MemEff>,
    pub way_buffer: Vec<Vec<Point>>,

    // Current visual types
    pub visual_parser: V,
    pub area_type: AreaVisualType,
    pub node_type: NodeVisualType,
    pub way_type: WayVisualType,
}

impl<P: Projection, V: VisualParser> WorldGenerator<P, V> {
    pub fn new(
        center: Point,
        (num_cols, num_rows): (usize, usize),
        zoom: u8,
        visual_parser: V,
        projection: P,
    ) -> Self {
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
                tiles.push(formats::MemEff::new(BBox {
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

            bbox,
            step: step_size,
            size: Vector2::new(num_cols as isize, num_rows as isize),
            tiles,

            way_buffer: vec![Vec::new(); num_cols * num_rows],
            visual_parser,
            area_type: AreaVisualType::None,
            node_type: NodeVisualType::None,
            way_type: WayVisualType::None,
        }
    }

    pub fn into_tiles(self) -> Vec<formats::MemEff> {
        self.tiles
    }

    fn flatten_index(&self, index: Index) -> Option<usize> {
        if index.x < 0 || self.size.x <= index.x || index.y < 0 || self.size.y <= index.y {
            None
        } else {
            Some((index.x + self.size.x * index.y) as usize)
        }
    }

    fn get_tile(&mut self, index: Index) -> Option<&mut formats::MemEff> {
        let index = self.flatten_index(index)?;
        self.tiles.get_mut(index)
    }

    fn get_wip_way(&mut self, index: Index) -> Option<&mut Vec<Point>> {
        let index = self.flatten_index(index)?;
        self.way_buffer.get_mut(index)
    }

    fn iter_nodes<'a>(&'_ self, nodes: &'a NodeRefList) -> impl Iterator<Item = Point> + 'a {
        let projection = self.projection;
        nodes
            .iter()
            .filter_map(move |node| projection.project(node))
    }
}

impl<P: Projection, V: VisualParser> Handler for WorldGenerator<P, V> {
    fn area(&mut self, area: &Area) {
        self.area_type = self.visual_parser.area(area.tags());
        if matches!(self.area_type, AreaVisualType::None) {
            return;
        }

        for ring in area.outer_rings() {
            let mut polygon: Vec<Point> = self.iter_nodes(ring).collect();

            // Collect the inner rings into reused Vecs
            let mut num_rings = 0;
            for inner_ring in area.inner_rings(ring) {
                // Reuse old Vec or push new one
                if num_rings < self.rings.len() {
                    self.rings[num_rings].clear();
                    let inner_ring = self.iter_nodes(inner_ring);
                    self.rings[num_rings].extend(inner_ring);
                } else {
                    self.rings.push(self.iter_nodes(inner_ring).collect());
                }

                // Only count non-empty rings
                if !self.rings[num_rings].is_empty() {
                    num_rings += 1;
                }
            }
            // Add the inner rings to the outer ring before clipping
            if num_rings > 0 {
                combine_rings(&mut polygon, &mut self.rings[0..num_rings]);
                log::info!(
                    "Combined {} inner rings @ {}",
                    num_rings,
                    area.original_id()
                );
            }

            self.clip_polygon(polygon);
        }
    }

    fn node(&mut self, _node: &Node) {}

    fn way(&mut self, way: &Way) {
        self.way_type = self.visual_parser.way(way.tags());
        if matches!(self.way_type, WayVisualType::None) {
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

        self.clip_path(self.iter_nodes(nodes));
    }
}

impl<P: Projection, V: VisualParser> Grid for WorldGenerator<P, V> {
    fn path_enter(&mut self, index: Index, point: Point) {
        if let Some(way) = self.get_wip_way(index) {
            assert_eq!(way.len(), 0);
            way.push(point);
        }
    }

    fn path_step(&mut self, index: Index, point: Point) {
        if let Some(way) = self.get_wip_way(index) {
            way.push(point);
        }
    }

    fn path_leave(&mut self, index: Index, point: Point) {
        let way_type = self.way_type;
        let Some(index) = self.flatten_index(index) else {return;};
        if let Some(way) = self.way_buffer.get_mut(index) {
            way.push(point);
            if let Some(tile) = self.tiles.get_mut(index) {
                tile.add_way(way, way_type);
            }
            way.clear();
        }
    }

    fn polygon_add(&mut self, index: Index, polygon: &[Point]) {
        let area_type = self.area_type;
        if let Some(tile) = self.get_tile(index) {
            if !polygon.is_empty() {
                tile.add_area(polygon, area_type);
            }
        }
    }

    fn point_add(&mut self, index: Index, point: Point) {
        let node_type = self.node_type;
        if let Some(tile) = self.get_tile(index) {
            tile.add_node(point, node_type);
        }
    }

    fn index_range(&self) -> Index {
        self.size
    }

    fn tile_box(&self, index: Index) -> BBox {
        let min = self.bbox.min + self.step.component_mul(&index.map(|i| i as f64));
        BBox {
            min,
            max: min + self.step,
        }
    }

    fn lookup_point(&self, point: Vector2<f64>) -> Index {
        (point - self.bbox.min)
            .component_div(&self.step)
            .map(|f| f.floor() as isize)
    }
}

