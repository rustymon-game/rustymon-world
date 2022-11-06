use libosmium::handler::Handler;
use libosmium::{Area, Node, Way};
use std::time::{Duration, Instant};

pub struct Timer<T: Handler> {
    handler: T,
    areas: (u32, Duration),
    nodes: (u32, Duration),
    ways: (u32, Duration),
}
impl<T: Handler> Timer<T> {
    pub fn wrap(handler: T) -> Self {
        Timer {
            handler,
            areas: (0, Duration::default()),
            nodes: (0, Duration::default()),
            ways: (0, Duration::default()),
        }
    }

    pub fn print(&self) {
        let areas = self.areas.1 / self.areas.0;
        let nodes = self.nodes.1 / self.nodes.0;
        let ways = self.ways.1 / self.ways.0;
        eprintln!("Areas: {:?}", areas);
        eprintln!("Nodes: {:?}", nodes);
        eprintln!("Ways: {:?}", ways);
    }

    pub fn unwrap(self) -> T {
        self.handler
    }
}

impl<T: Handler> Handler for Timer<T> {
    fn area(&mut self, area: &Area) {
        let now = Instant::now();
        self.handler.area(area);
        self.areas.0 += 1;
        self.areas.1 += now.elapsed();
    }

    fn node(&mut self, node: &Node) {
        let now = Instant::now();
        self.handler.node(node);
        self.nodes.0 += 1;
        self.nodes.1 += now.elapsed();
    }

    fn way(&mut self, way: &Way) {
        let now = Instant::now();
        self.handler.way(way);
        self.ways.0 += 1;
        self.ways.1 += now.elapsed();
    }
}
