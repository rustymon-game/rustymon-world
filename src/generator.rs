use crate::formats::Constructable;
use crate::geometry::bbox::GenericBox;
use crate::geometry::grid::{Grid, Index};
use crate::geometry::polygon::combine_rings;
use crate::geometry::{BBox, Point};
use libosmium::area::Area;
use libosmium::handler::Handler;
use libosmium::location::PRECISION;
use libosmium::node::{Node, NodeRef};
use libosmium::node_ref_list::NodeRefList;
use libosmium::way::Way;
use nalgebra::Vector2;

pub struct WorldGenerator<T: Constructable> {
    pub int_box: GenericBox<i32>,
    pub rings: Vec<Vec<Point>>,

    // Grid
    pub bbox: BBox,
    pub step: Vector2<f64>,
    pub size: Vector2<isize>,
    pub tiles: Vec<Construction<T>>,
}

impl<T: Constructable> WorldGenerator<T> {
    pub fn new(center: Point, step_num: (usize, usize), step_size: Point) -> WorldGenerator<T> {
        let min = Vector2::new(
            center.x - step_num.0 as f64 * step_size.x / 2.0,
            center.y - step_num.1 as f64 * step_size.y / 2.0,
        );

        let mut tiles = Vec::with_capacity(step_num.0 * step_num.1);
        for y in 0..step_num.1 {
            for x in 0..step_num.0 {
                let min = Vector2::new(
                    min.x + x as f64 * step_size.x,
                    min.y + y as f64 * step_size.y,
                );
                tiles.push(Construction {
                    constructing: T::new(BBox {
                        min,
                        max: min + step_size,
                    }),
                    wip_way: Vec::new(),
                });
            }
        }

        let bbox = BBox {
            min,
            max: Vector2::new(
                min.x + step_num.0 as f64 * step_size.x,
                min.y + step_num.1 as f64 * step_size.y,
            ),
        };

        WorldGenerator {
            int_box: GenericBox {
                min: bbox.min.map(|f| (f * PRECISION as f64).floor() as i32),
                max: bbox.max.map(|f| (f * PRECISION as f64).ceil() as i32),
            },
            rings: Vec::new(),

            bbox,
            step: step_size,
            size: Vector2::new(step_num.0 as isize, step_num.1 as isize),
            tiles,
        }
    }

    pub fn into_tiles(self) -> Vec<T> {
        self.tiles
            .into_iter()
            .map(|tile| tile.constructing)
            .collect()
    }

    fn get_tile(&mut self, index: Index) -> Option<&mut Construction<T>> {
        if index.x < 0 || self.size.x <= index.x || index.y < 0 || self.size.y <= index.y {
            return None;
        }
        self.tiles
            .get_mut((index.x + self.size.x * index.y) as usize)
    }
}

impl<T: Constructable> Handler for WorldGenerator<T> {
    fn area(&mut self, area: &Area) {
        for ring in area.outer_rings() {
            let mut polygon: Vec<Point> = nodes_to_iter(ring).collect();

            // Collect the inner rings into reused Vecs
            let mut num_rings = 0;
            for inner_ring in area.inner_rings(ring) {
                // Reuse old Vec or push new one
                if num_rings < self.rings.len() {
                    self.rings[num_rings].clear();
                    self.rings[num_rings].extend(nodes_to_iter(inner_ring));
                } else {
                    self.rings.push(nodes_to_iter(inner_ring).collect());
                }

                // Only count non-empty rings
                if self.rings[num_rings].len() > 0 {
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

    fn node(&mut self, node: &Node) {
        let location = node.location();
        if location.is_defined() && location.is_valid() {
            let point = Point::new(location.lon(), location.lat());
            self.clip_point(point);
        }
    }

    fn way(&mut self, way: &Way) {
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

        self.clip_path(nodes_to_iter(nodes));
    }
}

impl<T: Constructable> Grid for WorldGenerator<T> {
    fn path_enter(&mut self, index: Index, point: Point) {
        if let Some(tile) = self.get_tile(index) {
            assert_eq!(tile.wip_way.len(), 0);
            tile.wip_way.push(point);
        }
    }

    fn path_step(&mut self, index: Index, point: Point) {
        if let Some(tile) = self.get_tile(index) {
            tile.wip_way.push(point);
        }
    }

    fn path_leave(&mut self, index: Index, point: Point) {
        if let Some(tile) = self.get_tile(index) {
            tile.wip_way.push(point);
            tile.constructing.add_way(tile.wip_way.clone());
            tile.wip_way.clear();
        }
    }

    fn polygon_add(&mut self, index: Index, polygon: Vec<Point>) {
        if let Some(tile) = self.get_tile(index) {
            if polygon.len() > 0 {
                tile.constructing.add_area(polygon);
            }
        }
    }

    fn point_add(&mut self, index: Index, point: Point) {
        if let Some(tile) = self.get_tile(index) {
            tile.constructing.add_node(point);
        }
    }

    fn index_range(&self) -> Index {
        self.size
    }

    fn tile_box(&self, index: Vector2<isize>) -> BBox {
        let min = self.bbox.min + self.step.component_mul(&index.map(|i| i as f64));
        BBox {
            min,
            max: min + self.step,
        }
    }

    fn lookup_point(&self, point: Vector2<f64>) -> Vector2<isize> {
        (point - self.bbox.min)
            .component_div(&self.step)
            .map(|f| f.floor() as isize)
    }
}

pub struct Construction<T> {
    pub constructing: T,
    pub wip_way: Vec<Point>,
}

fn nodes_to_iter<'a>(ring: &'a NodeRefList) -> impl Iterator<Item = Point> + 'a {
    ring.iter()
        .map(NodeRef::get_location)
        .flatten()
        .map(|l| Point::new(l.lon(), l.lat()))
}
