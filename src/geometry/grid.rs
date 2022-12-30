use super::primitives::{Gt, HalfPlane, Line, Lt, X, Y};
use super::{bbox::BBox, Point};
use crate::geometry::bbox::GenericBox;
use nalgebra::Vector2;
use smallvec::SmallVec;

pub type Index = Vector2<isize>;
pub type IndexBox = GenericBox<isize>;

#[derive(Clone)]
pub struct Grid {
    /// The number of boxes in each direction
    boxes_num: Index,

    /// Size of each box
    boxes_size: Vector2<f64>,

    /// Box encompassing the whole grid
    boundary: BBox,

    /// Vectors to store partial paths in
    ///
    /// One for each tile.
    path_buffer: Vec<Vec<Point>>,
}
impl Grid {
    pub fn new(min: Vector2<f64>, boxes_num: Vector2<usize>, boxes_size: Vector2<f64>) -> Self {
        Self {
            boxes_num: Vector2::new(boxes_num.x as isize, boxes_num.y as isize),
            boxes_size,
            boundary: BBox {
                min,
                max: min
                    + Vector2::new(
                        boxes_num.x as f64 * boxes_size.x,
                        boxes_num.y as f64 * boxes_size.y,
                    ),
            },
            path_buffer: vec![Vec::new(); boxes_num.x * boxes_num.y],
        }
    }

    fn flatten_index(&self, index: Index) -> Option<usize> {
        if (0..self.boxes_num.x).contains(&index.x) && (0..self.boxes_num.y).contains(&index.y) {
            Some((index.x + self.boxes_num.x * index.y) as usize)
        } else {
            None
        }
    }

    fn get_partial_path(&mut self, index: Index) -> Option<&mut Vec<Point>> {
        let index = self.flatten_index(index)?;
        self.path_buffer.get_mut(index)
    }

    /// Push a point to a partial path
    #[inline]
    fn path_push(&mut self, index: Index, point: Point) {
        if let Some(path) = self.get_partial_path(index) {
            path.push(point);
        }
    }

    /// Push the final point to a partial path, publish and then empty it.
    fn path_publish(
        &mut self,
        index: Index,
        point: Point,
        publish: &mut impl FnMut(usize, &[Point]),
    ) {
        let Some(index) = self.flatten_index(index) else {return;};
        if let Some(path) = self.path_buffer.get_mut(index) {
            path.push(point);
            publish(index, path);
            path.clear();
        }
    }

    fn tile_box(&self, index: Vector2<isize>) -> BBox {
        let min = self.boundary.min + self.boxes_size.component_mul(&index.map(|i| i as f64));
        BBox {
            min,
            max: min + self.boxes_size,
        }
    }

    fn lookup_point(&self, point: Vector2<f64>) -> Vector2<isize> {
        (point - self.boundary.min)
            .component_div(&self.boxes_size)
            .map(|f| f.floor() as isize)
    }

    pub fn clip_polygon(&mut self, polygon: Vec<Point>, mut publish: impl FnMut(usize, &[Point])) {
        let size = self.boxes_num;

        let mut index_box =
            IndexBox::from_iter(polygon.iter().map(|&point| self.lookup_point(point)));

        // Polygon is already contained in a single tile
        if index_box.min == index_box.max {
            if let Some(index) = self.flatten_index(index_box.min) {
                publish(index, &polygon);
            }
            return;
        }

        // Fix the the polygon's box to actually contain it
        index_box.max += Index::new(1, 1);

        // Polygon is actually outside of this grid
        if index_box.min.x >= size.x
            || index_box.min.y >= size.y
            || index_box.max.x < 0
            || index_box.max.y < 0
        {
            return;
        }

        // Clip the polygon's bounding box such that it can be used as range to iterate over
        if index_box.min.x < 0 {
            index_box.min.x = 0;
        }
        if index_box.min.y < 0 {
            index_box.min.y = 0;
        }
        if index_box.max.x > size.x {
            index_box.max.x = size.x;
        }
        if index_box.max.y > size.y {
            index_box.max.y = size.x;
        }

        // Three reusable vectors for the clipping process
        let mut temp = Vec::new();
        let mut row = Vec::new();
        let mut tile = Vec::new();

        for y in index_box.min.y..index_box.max.y {
            let bbox = self.tile_box(Index::new(0, y));

            temp.clear();
            HalfPlane(Y, Gt, bbox.min.y).clip(&polygon, &mut temp);
            row.clear();
            HalfPlane(Y, Lt, bbox.max.y).clip(&temp, &mut row);

            for x in index_box.min.x..index_box.max.x {
                let index = Index::new(x, y);
                let bbox = self.tile_box(index);

                temp.clear();
                HalfPlane(X, Gt, bbox.min.x).clip(&row, &mut temp);
                tile.clear();
                HalfPlane(X, Lt, bbox.max.x).clip(&temp, &mut tile);

                if let Some(index) = self.flatten_index(index) {
                    publish(index, &tile);
                }
            }
        }
    }

    pub fn clip_path(
        &mut self,
        path: impl Iterator<Item = Point>,
        mut publish: impl FnMut(usize, &[Point]),
    ) {
        let mut path = path;

        let mut current_p = if let Some(point) = path.next() {
            point
        } else {
            return;
        };
        let mut current_i = self.lookup_point(current_p);

        self.path_push(current_i, current_p);

        for next_p in path {
            let next_i = self.lookup_point(next_p);

            while next_i != current_i {
                // Get all intersections which with the current box in towards the next point
                let mut intersections: SmallVec<[(Index, Point); 2]> = SmallVec::new();
                let current_box = self.tile_box(current_i);
                if next_i.x > current_i.x {
                    intersections.push((
                        Index::new(1, 0),
                        Line(X, current_box.max.x).intersect(current_p, next_p),
                    ));
                }
                if next_i.x < current_i.x {
                    intersections.push((
                        Index::new(-1, 0),
                        Line(X, current_box.min.x).intersect(current_p, next_p),
                    ));
                }
                if next_i.y > current_i.y {
                    intersections.push((
                        Index::new(0, 1),
                        Line(Y, current_box.max.y).intersect(current_p, next_p),
                    ));
                }
                if next_i.y < current_i.y {
                    intersections.push((
                        Index::new(0, -1),
                        Line(Y, current_box.min.y).intersect(current_p, next_p),
                    ));
                }

                // Select nearest intersection
                let (delta_i, intersection) = match intersections.as_slice() {
                    [] => unreachable!("The loop condition implies at least on if branch"),
                    [tuple] => tuple,
                    [tuple_1 @ (_, point_1), tuple_2 @ (_, point_2)] => {
                        if (current_p - point_1).norm_squared()
                            < (current_p - point_2).norm_squared()
                        {
                            tuple_1
                        } else {
                            tuple_2
                        }
                    }
                    _ => unreachable!("Only two if branches can happen"),
                };

                // Step to this intersection and continue from there
                self.path_publish(current_i, *intersection, &mut publish);
                current_i += delta_i;
                self.path_push(current_i, *intersection);
            }
            self.path_push(next_i, next_p);

            current_p = next_p;
            current_i = self.lookup_point(next_p);
        }

        self.path_publish(current_i, current_p, &mut publish);
    }

    pub fn clip_point(&mut self, point: Point, mut publish: impl FnMut(usize, Point)) {
        let index = self.lookup_point(point);
        if let Some(index) = self.flatten_index(index) {
            publish(index, point);
        }
    }
}
