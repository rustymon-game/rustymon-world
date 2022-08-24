use super::primitives::{Gt, HalfPlane, Line, Lt, X, Y};
use super::{bbox::BBox, Point};
use crate::geometry::bbox::GenericBox;
use nalgebra::Vector2;
use smallvec::SmallVec;

pub type Index = Vector2<isize>;
pub type IndexBox = GenericBox<isize>;

pub trait Grid {
    fn clip_polygon(&mut self, polygon: Vec<Point>) {
        let size = self.index_range();

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

                self.polygon_add(index, Vec::from(tile.as_slice()));
            }
        }
    }

    fn clip_path(&mut self, path: impl Iterator<Item = Point>) {
        let mut path = path;

        let mut current_p = if let Some(point) = path.next() {
            point
        } else {
            return;
        };
        let mut current_i = self.lookup_point(current_p);

        self.path_enter(current_i, current_p);

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

    fn clip_point(&mut self, point: Point) {
        let index = self.lookup_point(point);
        self.point_add(index, point);
    }

    fn path_enter(&mut self, index: Index, point: Point);
    fn path_step(&mut self, index: Index, point: Point);
    fn path_leave(&mut self, index: Index, point: Point);

    fn polygon_add(&mut self, index: Index, polygon: Vec<Point>);

    fn point_add(&mut self, index: Index, point: Point);

    /// Get the excluded upper bound for indexes
    ///
    /// Used in [`clip_polygon`] to minimize computation
    fn index_range(&self) -> Index;
    /// Grid index to BBox i.e. points
    fn tile_box(&self, index: Vector2<isize>) -> BBox;
    /// Point to grid index
    fn lookup_point(&self, point: Vector2<f64>) -> Vector2<isize>;
}
