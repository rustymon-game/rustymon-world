use super::primitives::{Gt, HalfPlane, Line, Lt, X, Y};
use super::{bbox::BBox, Point};
use crate::geometry::bbox::GenericBox;
use nalgebra::Vector2;
use smallvec::SmallVec;

pub struct Grid<T: GridTile> {
    pub bbox: BBox,
    pub step: Vector2<f64>,
    size: Vector2<isize>,
    pub tiles: Vec<T>,
}

pub trait GridTile {
    fn new(bbox: BBox) -> Self;
    fn path_enter(&mut self, point: Point);
    fn path_step(&mut self, point: Point);
    fn path_leave(&mut self, point: Point);
    fn polygon_add(&mut self, polygon: Vec<Point>);
    fn point_add(&mut self, point: Point);
}

pub type Index = Vector2<isize>;
pub type IndexBox = GenericBox<isize>;

impl<T: GridTile> Grid<T> {
    pub fn new(center: Vector2<f64>, step_num: (usize, usize), step_size: Vector2<f64>) -> Grid<T> {
        let min = Vector2::new(
            center.x - step_num.0 as f64 * step_size.x / 2.0,
            center.y - step_num.1 as f64 * step_size.y / 2.0,
        );

        let mut boxes = Vec::with_capacity(step_num.0 * step_num.1);
        for y in 0..step_num.1 {
            for x in 0..step_num.0 {
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

        Grid {
            bbox: BBox {
                min,
                max: boxes.last().unwrap().max,
            },
            step: step_size,
            size: Index::new(step_num.0 as isize, step_num.1 as isize),
            tiles: boxes.into_iter().map(T::new).collect(),
        }
    }

    pub fn clip_polygon<I: IntoIterator<Item = Point>>(&mut self, polygon: I) {
        let polygon: Vec<Point> = polygon.into_iter().collect();

        let mut index_box =
            IndexBox::from_iter(polygon.iter().map(|&point| self.lookup_point(point)));

        // Polygon is already contained in a single tile
        if index_box.min == index_box.max {
            self.polygon_add(index_box.min, polygon);
            return;
        }

        // Fix the the polygon's box to actually contain it
        index_box.max += Index::new(1, 1);

        // Polygon is actually outside of this grid
        if index_box.min.x >= self.size.x
            || index_box.min.y >= self.size.y
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
        if index_box.max.x > self.size.x {
            index_box.max.x = self.size.x;
        }
        if index_box.max.y > self.size.y {
            index_box.max.y = self.size.x;
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

                self.polygon_add(index, Vec::from(tile.as_slice()));
            }
        }
    }

    pub fn clip_path<I: IntoIterator<Item = Point>>(&mut self, path: I) {
        /*
        let mut path: impl Iterator<Item = Vector2<f64>> = path.into_iter();
        */
        let mut path = path.into_iter();

        let mut current_p = if let Some(point) = path.next() {
            point
        } else {
            return;
        };
        let mut current_i = self.lookup_point(current_p);

        if self.bbox.contains(current_p) {
            self.path_enter(current_i, current_p);
        }

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
                    &[] => unreachable!("The loop condition implies at least on if branch"),
                    &[tuple] => tuple,
                    &[tuple_1 @ (_, point_1), tuple_2 @ (_, point_2)] => {
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
                self.path_leave(current_i, intersection);
                current_i += delta_i;
                self.path_enter(current_i, intersection);
            }
            self.path_step(next_i, next_p);

            current_p = next_p;
            current_i = self.lookup_point(next_p);
        }

        self.path_leave(current_i, current_p);
    }

    pub fn clip_point(&mut self, point: Point) {
        let index = self.lookup_point(point);
        self.point_add(index, point);
    }

    fn safe_vector_index(&self, index: Index) -> Option<usize> {
        if 0 <= index.x && index.x < self.size.x && 0 <= index.y && index.y < self.size.y {
            Some((index.x + self.size.x * index.y) as usize)
        } else {
            None
        }
    }

    fn path_enter(&mut self, index: Index, point: Point) {
        if let Some(index) = self.safe_vector_index(index) {
            self.tiles[index].path_enter(point);
        }
    }

    fn path_step(&mut self, index: Index, point: Point) {
        if let Some(index) = self.safe_vector_index(index) {
            self.tiles[index].path_step(point);
        }
    }

    fn path_leave(&mut self, index: Index, point: Point) {
        if let Some(index) = self.safe_vector_index(index) {
            self.tiles[index].path_leave(point);
        }
    }

    fn polygon_add(&mut self, index: Index, polygon: Vec<Point>) {
        if let Some(index) = self.safe_vector_index(index) {
            self.tiles[index].polygon_add(polygon);
        }
    }

    fn point_add(&mut self, index: Index, point: Point) {
        if let Some(index) = self.safe_vector_index(index) {
            self.tiles[index].point_add(point);
        }
    }

    pub fn tile_box(&self, index: Vector2<isize>) -> BBox {
        let min = self.bbox.min + self.step.component_mul(&index.map(|i| i as f64));
        BBox {
            min,
            max: min + self.step,
        }
    }

    pub fn lookup_point(&self, point: Vector2<f64>) -> Vector2<isize> {
        (point - self.bbox.min)
            .component_div(&self.step)
            .map(|f| f.floor() as isize)
    }
}
