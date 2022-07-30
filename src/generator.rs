use crate::formats::Constructable;
use crate::geometry::grid::{Grid, GridTile};
use crate::geometry::{BBox, Point};
use osmium::area::Area;
use osmium::handler::Handler;
use osmium::node::{Node, NodeRef};
use osmium::way::Way;

pub type WorldGenerator<T> = Grid<Construction<T>>;

impl<T: Constructable> Handler for WorldGenerator<T> {
    fn area(&mut self, area: &Area) {
        for tile in self.tiles.iter_mut() {
            for ring in area.outer_rings() {
                let polygon = ring
                    .iter()
                    .map(NodeRef::get_location)
                    .flatten()
                    .map(|l| Point::new(l.lon(), l.lat()));

                let polygon = tile.bbox.clip_polygon(polygon);

                if polygon.len() > 0 {
                    tile.constructing.add_area(polygon);
                }
            }
        }
    }

    fn node(&mut self, node: &Node) {
        for tile in self.tiles.iter_mut() {
            let location = node.location();
            if location.is_defined() && location.is_valid() {
                let point = Point::new(location.lon(), location.lat());
                if tile.bbox.contains(point) {
                    tile.constructing.add_node(point);
                }
            }
        }
    }

    fn way(&mut self, way: &Way) {
        let nodes = way.nodes();
        match (nodes.first(), nodes.last()) {
            (Some(first), Some(last)) => {
                if first.id == last.id {
                    return;
                }
            }
            _ => return,
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
}
