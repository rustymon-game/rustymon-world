use std::{panic, thread};

use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use libosmium::{Area, Handler, Item, ItemBuffer, ItemRef, Node, Way};
use log::{debug, error};

use crate::features::FeatureParser;
use crate::formats::Tile;
use crate::generator::WorldGenerator;
use crate::projection::Projection;

/// Bytes size of buffer
/// - bigger consumes more memory
/// - lower produces more synchronization overhead
pub const CAPACITY: usize = 2 << 20;

pub struct MultithreadedGenerator<P: Projection, V: FeatureParser> {
    buffer: ItemBuffer,
    sender: Sender<ItemBuffer>,

    /// "Empty" world generator to clone for the threads
    generator: WorldGenerator<P, V>,
    /// Receiver part of channel to clone for the threads
    receiver: Receiver<ItemBuffer>,

    /// Join handles for the worker threads
    handles: Vec<thread::JoinHandle<Vec<Tile<V::Feature>>>>,
}

impl<P: Projection, V: FeatureParser> MultithreadedGenerator<P, V>
where
    V: Clone + Send + 'static,
    V::Feature: Clone + Send + 'static,
{
    /// Wrap a [WorldGenerator] to be multithreaded
    pub fn new(generator: WorldGenerator<P, V>) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            buffer: ItemBuffer::with_capacity(CAPACITY),
            sender,

            generator,
            receiver,

            handles: Vec::new(),
        }
    }

    /// Spawn worker threads
    pub fn spawn_workers(&mut self, worker: usize) {
        for i in 0..worker {
            let mut generator = self.generator.clone();
            let receiver = self.receiver.clone();
            let handle = thread::spawn(move || {
                while let Ok(buffer) = receiver.recv() {
                    debug!(
                        "Worker {} received ItemBuffer: {} remaining",
                        i,
                        receiver.len()
                    );
                    for item in buffer.iter() {
                        match item.cast() {
                            Some(ItemRef::Area(area)) => generator.area(area),
                            Some(ItemRef::Node(node)) => generator.node(node),
                            Some(ItemRef::Way(way)) => generator.way(way),
                            _ => {
                                error!(
                                    "The buffer contains an invalid item: {:?}",
                                    item.item_type()
                                );
                            }
                        }
                    }
                }
                generator.into_tiles()
            });
            self.handles.push(handle);
            debug!("Spawned a worker {}", i);
        }
    }

    /// Handle any osm item by populating the buffer.
    pub fn handle(&mut self, item: &impl AsRef<Item>) -> Result<(), SendError<ItemBuffer>> {
        if self.buffer.fits(item) || self.buffer.is_empty() {
            self.buffer.push(item);
        } else {
            self.sender.send(std::mem::replace(
                &mut self.buffer,
                ItemBuffer::with_capacity(CAPACITY),
            ))?;
            debug!(
                "Send ItemBuffer to workers: {} in channel",
                self.sender.len()
            );
        }
        Ok(())
    }

    /// Join all workers and collect their tiles
    pub fn into_tiles(mut self) -> Vec<Tile<V::Feature>> {
        drop(self.sender);
        for handle in self.handles {
            let tiles = match handle.join() {
                Ok(tiles) => tiles,
                Err(error) => panic::resume_unwind(error),
            };
            for (i, from) in tiles.into_iter().enumerate() {
                if let Some(to) = self.generator.tiles.get_mut(i) {
                    to.areas.extend(from.areas.into_iter());
                    to.nodes.extend(from.nodes.into_iter());
                    to.ways.extend(from.ways.into_iter());
                } else {
                    error!("A worker contains tiles the base doesn't!");
                }
            }
        }
        self.generator.tiles
    }
}

impl<P: Projection, V: FeatureParser> Handler for MultithreadedGenerator<P, V>
where
    V: Clone + Send + 'static,
    V::Feature: Clone + Send + 'static,
{
    fn area(&mut self, area: &Area) {
        self.handle(area).unwrap();
    }

    fn node(&mut self, node: &Node) {
        self.handle(node).unwrap();
    }

    fn way(&mut self, way: &Way) {
        self.handle(way).unwrap();
    }
}
