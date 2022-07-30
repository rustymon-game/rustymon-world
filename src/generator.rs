use crate::{geometry, BBox, Constructable};
use nalgebra::Vector2;
use osmium::area::Area;
use osmium::handler::Handler;
use osmium::node::{Node, NodeRef};
use osmium::way::Way;

pub type WorldGenerator<T> = MultiHandler<TileGenerator<T>>;
impl<T: Constructable> WorldGenerator<T> {
    pub fn new(
        center: Vector2<f64>,
        step_num: (usize, usize),
        step_size: Vector2<f64>,
    ) -> WorldGenerator<T> {
        let min = Vector2::new(
            center.x - step_num.0 as f64 * step_size.x / 2.0,
            center.y - step_num.1 as f64 * step_size.y / 2.0,
        );

        let mut boxes = Vec::with_capacity(step_num.0 * step_num.1);
        for x in 0..step_num.0 {
            for y in 0..step_num.1 {
                let min = Vector2::new(
                    min.x + x as f64 * step_size.x,
                    min.y + y as f64 * step_size.y,
                );
                boxes.push(BBox {
                    min,
                    max: min + step_size,
                });
            }
        }

        WorldGenerator {
            handlers: boxes.into_iter().map(|b| TileGenerator::new(b)).collect(),
        }
    }
}

pub struct MultiHandler<T: Handler> {
    pub handlers: Vec<T>,
}
impl<T: Handler> From<Vec<T>> for MultiHandler<T> {
    fn from(handlers: Vec<T>) -> Self {
        MultiHandler { handlers }
    }
}
impl<T: Handler + Send> Handler for MultiHandler<T> {
    fn area(&mut self, area: &Area) {
        self.handlers.iter_mut().for_each(|h| h.area(area));
    }
    fn node(&mut self, node: &Node) {
        self.handlers.iter_mut().for_each(|h| h.node(node));
    }
    fn way(&mut self, way: &Way) {
        self.handlers.iter_mut().for_each(|h| h.way(way));
    }
}

pub struct TileGenerator<T: Constructable> {
    /// Bounding box to clip everything to
    pub bbox: BBox,

    /// The output format's instance which is being constructed
    pub constructing: T,
}
unsafe impl<T: Constructable> Send for TileGenerator<T> {}
impl<T: Constructable> TileGenerator<T> {
    pub fn new(bbox: BBox) -> TileGenerator<T> {
        TileGenerator {
            bbox,
            constructing: T::new(bbox),
        }
    }
}
impl<T: Constructable> Handler for TileGenerator<T> {
    fn area(&mut self, area: &Area) {
        for ring in area.outer_rings() {
            let polygon = ring
                .iter()
                .map(NodeRef::get_location)
                .flatten()
                .map(|l| Vector2::new(l.lon(), l.lat()));

            let polygon = Vec::from_iter(geometry::iter::clip_polygon(polygon, self.bbox));

            if polygon.len() > 0 {
                self.constructing.add_area(polygon);
            }
        }
    }

    fn node(&mut self, node: &Node) {
        let location = node.location();
        if location.is_defined() && location.is_valid() {
            let point = Vector2::new(location.lon(), location.lat());
            if self.bbox.contains(point) {
                self.constructing.add_node(point);
            }
        }
    }

    fn way(&mut self, way: &Way) {
        let path = way
            .nodes()
            .iter()
            .map(NodeRef::get_location)
            .flatten()
            .map(|l| Vector2::new(l.lon(), l.lat()));
        self.constructing.extend_ways(self.bbox.clip_path(path));
    }
}
