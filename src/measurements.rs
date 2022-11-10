//! Collection of measurements useful for debugging and "primitive benchmarking"
use std::fmt::Debug;
use std::ops::AddAssign;
use std::time::{Duration, Instant};

use libosmium::{Area, Handler, Node, Way};

/// Generates the fields and methods common to any measured handler
macro_rules! measured_handler {
    ($(#[doc = $doc:literal])* pub struct $name:ident<H: Handler> {$T:ty}) => {
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

            /// Print the measured results
            pub fn print(&self) {
                let CombinedMeasure { areas, nodes, ways } = &self.measurements;
                for (name, measurement) in [("Areas", areas), ("Nodes", nodes), ("Ways", ways)] {
                    Self::print_single(name, measurement);
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
    pub struct TimedHandler<H: Handler> {Duration}
}
impl<H: Handler> TimedHandler<H> {
    fn print_single(name: &str, measurement: &Measurement<Duration>) {
        eprintln!(
            "{} {name} took {:?} to process at {:?} each",
            measurement.number,
            measurement.acc,
            measurement.avg()
        );
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

measured_handler! {
    /// counts the number of tags on per object.
    pub struct TagCountedHandler<H: Handler> {u32}
}
impl<H: Handler> Handler for TagCountedHandler<H> {
    fn area(&mut self, area: &Area) {
        self.measurements
            .areas
            .add(area.tags().into_iter().count() as u32);
        self.handler.area(area);
    }

    fn node(&mut self, node: &Node) {
        self.measurements
            .nodes
            .add(node.tags().into_iter().count() as u32);
        self.handler.node(node);
    }

    fn way(&mut self, way: &Way) {
        self.measurements
            .ways
            .add(way.tags().into_iter().count() as u32);
        self.handler.way(way);
    }
}
impl<H: Handler> TagCountedHandler<H> {
    fn print_single(name: &str, measurement: &Measurement<u32>) {
        eprintln!(
            "{name} have between {} and {} tags, averaging at {}",
            measurement.min,
            measurement.max,
            measurement.avg()
        );
    }
}

/// Collective measurements of a single type seperated in areas, nodes and ways.
#[derive(Default, Copy, Clone, Debug)]
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
    pub fn avg(&self) -> T::Avg {
        self.acc.avg(self.number)
    }
}
// Custom Debug impl
// - also outputs `avg`
impl<T: Measureable> Debug for Measurement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Measurement")
            .field("number", &self.number)
            .field("acc", &self.acc)
            .field("min", &self.min)
            .field("max", &self.max)
            .field("avg", &self.avg())
            .finish()
    }
}

/// Empty trait combining all traits required of measured values into a single shorthand.
pub trait Measureable: Default + Copy + PartialOrd + AddAssign<Self> + Debug {
    type Avg: Debug;
    fn avg(self, count: u32) -> Self::Avg;
}
impl Measureable for Duration {
    type Avg = Self;
    fn avg(self, count: u32) -> Self::Avg {
        self / count as u32
    }
}
macro_rules! impl_numeric {
    ($($T:ty),*) => {
        $(
            impl Measureable for $T {
                type Avg = f64;
                fn avg(self, count: u32) -> Self::Avg {
                    self as f64 / count as f64
                }
            }
        )*
    };
}
impl_numeric!(u32, usize);
