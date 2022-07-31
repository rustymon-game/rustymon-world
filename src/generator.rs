use crate::formats::Constructable;
use crate::geometry::bbox::GenericBox;
use crate::geometry::grid::{Grid, GridTile};
use crate::geometry::{BBox, Point};
use nalgebra::Vector2;
use osmium::area::Area;
use osmium::handler::Handler;
use osmium::location::PRECISION;
use osmium::node::{Node, NodeRef};
use osmium::node_ref_list::NodeRefList;
use osmium::way::Way;
use std::ops::{Deref, DerefMut};

pub struct WorldGenerator<T: Constructable> {
    pub grid: Grid<Construction<T>>,
    pub int_box: GenericBox<i32>,
}
impl<T: Constructable> Deref for WorldGenerator<T> {
    type Target = Grid<Construction<T>>;

    fn deref(&self) -> &Self::Target {
        &self.grid
    }
}
impl<T: Constructable> DerefMut for WorldGenerator<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.grid
    }
}

impl<T: Constructable> WorldGenerator<T> {
    pub fn new(center: Point, step_num: (usize, usize), step_size: Point) -> WorldGenerator<T> {
        let grid = Grid::new(center, step_num, step_size);
        WorldGenerator {
            int_box: GenericBox {
                min: (&grid.bbox.min).map(|f| (f * PRECISION as f64).floor() as i32),
                max: (&grid.bbox.max).map(|f| (f * PRECISION as f64).ceil() as i32),
            },
            grid,
        }
    }

    pub fn into_tiles(self) -> Vec<T> {
        self.grid
            .tiles
            .into_iter()
            .map(|tile| tile.constructing)
            .collect()
    }
}

impl<T: Constructable> Handler for WorldGenerator<T> {
    fn area(&mut self, area: &Area) {
        for ring in area.outer_rings() {
            // Skip rings which don't touch the grid
            if !quick_check_points(self.int_box, ring) {
                continue;
            }

            let polygon = ring
                .iter()
                .map(NodeRef::get_location)
                .flatten()
                .map(|l| Point::new(l.lon(), l.lat()));
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

        // Skip ways which don't touch the grid
        if !quick_check_points(self.int_box, nodes) {
            return;
        }

        let path = way
            .nodes()
            .iter()
            .map(NodeRef::get_location)
            .flatten()
            .map(|l| Point::new(l.lon(), l.lat()));

        self.clip_path(path);
    }
}

/// Check if at least any point lies in the bounding box
fn quick_check_points(bbox: GenericBox<i32>, points: &NodeRefList) -> bool {
    for point in points.iter() {
        // Since this is only a quick check if at least any point lies in the grid,
        // it doesn't matter if the point isn't actually initialized.
        // Any uninitialized point won't lie in the grid because osmium sets it to i32::MAX.
        let location = unsafe { point.location.assume_init() };
        if bbox.contains(Vector2::new(location.raw_x, location.raw_y)) {
            return true;
        }
    }
    return false;
}

pub struct Construction<T> {
    pub bbox: BBox,
    pub constructing: T,
    pub wip_way: Vec<Point>,
}
impl<T: Constructable> GridTile for Construction<T> {
    fn new(bbox: BBox) -> Self {
        Construction {
            bbox,
            constructing: T::new(bbox),
            wip_way: Vec::new(),
        }
    }
    fn path_enter(&mut self, point: Point) {
        assert_eq!(self.wip_way.len(), 0);
        self.wip_way.push(point);
    }
    fn path_step(&mut self, point: Point) {
        self.wip_way.push(point);
    }
    fn path_leave(&mut self, point: Point) {
        self.wip_way.push(point);
        self.constructing.add_way(self.wip_way.clone());
        self.wip_way.clear();
    }
    fn polygon_add(&mut self, polygon: Vec<Point>) {
        if polygon.len() > 0 {
            self.constructing.add_area(polygon);
        }
    }
    fn point_add(&mut self, point: Point) {
        self.constructing.add_node(point);
    }
}
