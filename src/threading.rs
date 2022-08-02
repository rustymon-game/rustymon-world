#![allow(dead_code)]
use libosmium::area::Area;
use libosmium::handler::Handler;
use libosmium::item::{Item, ItemRef};
use libosmium::node::Node;
use libosmium::way::Way;
use rayon::prelude::*;
use std::mem::transmute;
use std::slice;

const MB: usize = 1024 * 1024;

pub type ThreadedHandler<T> = BufferedHandler<HandlerCollection<T>>;
impl<T: Handler + Send> ThreadedHandler<T> {
    pub fn from_handlers(handlers: Vec<T>) -> Self {
        BufferedHandler::new(HandlerCollection(handlers))
    }

    pub fn into_handlers(self) -> Vec<T> {
        self.flusher.0
    }
}

pub struct HandlerCollection<T: Handler>(Vec<T>);
impl<T: Handler + Send> Flush for HandlerCollection<T> {
    fn flush(&mut self, buffer: &Buffer) {
        self.0.par_iter_mut().for_each(|handler| {
            for item in buffer.into_iter() {
                match item {
                    ItemRef::Area(area) => handler.area(area),
                    ItemRef::Node(node) => handler.node(node),
                    ItemRef::Way(way) => handler.way(way),
                    _ => {} // TODO
                }
            }
        });
    }
}

pub struct BufferedHandler<T: Flush> {
    buffer: Buffer,
    flusher: T,
}
pub trait Flush {
    fn flush(&mut self, buffer: &Buffer);
}
impl<T: Fn(&Buffer)> Flush for Box<T> {
    fn flush(&mut self, buffer: &Buffer) {
        self(buffer);
    }
}
impl<T: Flush> BufferedHandler<T> {
    pub fn new(flusher: T) -> Self {
        BufferedHandler {
            buffer: Buffer::new(),
            flusher,
        }
    }

    fn flush(&mut self) {
        self.flusher.flush(&self.buffer);
        self.buffer.clear();
    }

    fn push(&mut self, item: &Item) {
        self.buffer.push(item);

        // Fill the buffer to somewhere under a gigabyte
        if self.buffer.0.len() > 1000 * MB {
            self.flush();
        }
    }
}
impl<T: Flush> Handler for BufferedHandler<T> {
    fn area(&mut self, area: &Area) {
        self.push(area.as_ref())
    }
    fn node(&mut self, node: &Node) {
        self.push(node.as_ref())
    }
    fn way(&mut self, way: &Way) {
        self.push(way.as_ref())
    }
    fn flush(&mut self) {
        self.flush();
    }
}

pub struct Buffer(Vec<u8>);
impl Buffer {
    pub fn new() -> Buffer {
        Buffer(Vec::new())
    }
    pub fn push(&mut self, item: &Item) {
        self.0.extend_from_slice(unsafe {
            slice::from_raw_parts(transmute::<_, *mut u8>(item), item.byte_size() as usize)
        });
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<'a> IntoIterator for &'a Buffer {
    type Item = ItemRef<'a>;
    type IntoIter = BufferIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator {
            next: 0,
            buffer: self,
        }
    }
}
pub struct BufferIterator<'a> {
    next: usize,
    buffer: &'a Buffer,
}
impl<'a> Iterator for BufferIterator<'a> {
    type Item = ItemRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.next < self.buffer.0.len() {
                let next = self.buffer.0.as_ptr() as usize + self.next;
                let next = next as *const Item;
                let next = next.as_ref().expect("The Vec's alloc must be > 0");
                self.next += next.byte_size() as usize;
                Some(next.parse().unwrap())
            } else {
                None
            }
        }
    }
}
