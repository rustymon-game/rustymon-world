//! Collection of measurements useful for debugging and "primitive benchmarking"

use libosmium::{Area, Handler, Node, Way};
use std::ops::{AddAssign, Div};
use std::time::{Duration, Instant};

/// Generates the fields and methods common to any measured handler
macro_rules! measured_handler {
    ($(#[doc = $doc:literal])* pub struct $name:ident<$T:ty>) => {
        $(#[doc = $doc])*
        pub struct $name<H: Handler> {
            /// [Handler] to be measured
            pub handler: H,

            /// Collected measurements
            pub measurements: CombinedMeasure<$T>,
        }
        impl<H: Handler> $name<H> {
            /// Wrap a handler
            pub fn new(handler: H) -> Self {
                Self {
                    handler,
                    measurements: Default::default(),
                }
            }

            /// Unwrap the measured handler, dropping the measurements
            pub fn into_handler(self) -> H {
                self.handler
            }
        }
    };
}

measured_handler! {
    /// measures the time required to process the various types.
    pub struct TimedHandler<Duration>
}
impl<H: Handler> TimedHandler<H> {
    pub fn print(&self) {
        let CombinedMeasure { areas, nodes, ways } = &self.measurements;
        eprintln!("Areas: #{} @ {:?} each", areas.number, areas.avg());
        eprintln!("Nodes: #{} @ {:?} each", nodes.number, nodes.avg());
        eprintln!("Ways: #{} @ {:?} each", ways.number, ways.avg());
    }
}
impl<H: Handler> Handler for TimedHandler<H> {
    fn area(&mut self, area: &Area) {
        let now = Instant::now();
        self.handler.area(area);
        self.measurements.areas.add(now.elapsed());
    }

    fn node(&mut self, node: &Node) {
        let now = Instant::now();
        self.handler.node(node);
        self.measurements.nodes.add(now.elapsed());
    }

    fn way(&mut self, way: &Way) {
        let now = Instant::now();
        self.handler.way(way);
        self.measurements.ways.add(now.elapsed());
    }
}

/// Collective measurements of a single type seperated in areas, nodes and ways.
#[derive(Default)]
pub struct CombinedMeasure<T: Measureable> {
    pub areas: Measurement<T>,
    pub nodes: Measurement<T>,
    pub ways: Measurement<T>,
}

#[derive(Default, Copy, Clone)]
pub struct Measurement<T: Measureable> {
    /// How many values have been:
    /// - accumulated into `acc`
    /// - compared with `max`
    pub number: u32,

    /// Sum of all seen values
    pub acc: T,

    /// The lowest of all seen values
    pub min: T,

    /// The highest of all seen values
    pub max: T,
}
impl<T: Measureable> Measurement<T> {
    /// Add a data point to the measurement
    pub fn add(&mut self, value: T) {
        self.number += 1;
        self.acc += value;

        if self.number == 1 {
            // If this is the first value, set it as min and max
            // Since Measure<T> is generic, min and max can't be initialised with some MAX and MIN constants.
            self.min = value;
            self.max = value;
        } else {
            // Compare and set
            if self.max < value {
                self.max = value;
            }
            if self.min > value {
                self.min = value;
            }
        }
    }

    /// Get the average value
    pub fn avg(&self) -> <T as Div<u32>>::Output
    where
        T: Div<u32>,
    {
        self.acc / self.number
    }
}

/// Empty trait combining all traits required of measured values into a single shorthand.
pub trait Measureable: Default + Copy + PartialOrd + AddAssign<Self> {}
impl<T: Default + Copy + PartialOrd + AddAssign<T>> Measureable for T {}

pub struct Time;
pub struct TagCount;
