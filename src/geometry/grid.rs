use super::{bbox::BBox, Line, Point};
use nalgebra::Vector2;
use smallvec::SmallVec;

pub struct Grid<T: GridTile> {
    pub bbox: BBox,
    pub step: Vector2<f64>,
    pub size: Vector2<isize>,
    pub tiles: Vec<T>,
}

pub trait GridTile {
    fn path_enter(&mut self, point: Point);
    fn path_step(&mut self, point: Point);
    fn path_leave(&mut self, point: Point);
}

type Index = Vector2<isize>;

impl<T: GridTile> Grid<T> {
    pub fn new(bbox: BBox, step: Vector2<f64>) -> Grid<T> {
        let mut grid = Grid {
            bbox,
            step,
            tiles: Vec::new(),
            size: Vector2::zeros(),
        };
        grid.size = grid.lookup_point(grid.bbox.max);
        grid.tiles = Vec::with_capacity((grid.size.x * grid.size.y) as usize);
        grid
    }

    pub fn clip_path<I: IntoIterator<Item = Vector2<f64>>>(&mut self, path: I) {
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
                        Line::Vertical {
                            x: current_box.max.x,
                        }
                        .intersect(current_p, next_p),
                    ));
                }
                if next_i.x < current_i.x {
                    intersections.push((
                        Index::new(-1, 0),
                        Line::Vertical {
                            x: current_box.min.x,
                        }
                        .intersect(current_p, next_p),
                    ));
                }
                if next_i.y > current_i.y {
                    intersections.push((
                        Index::new(0, 1),
                        Line::Horizontal {
                            y: current_box.max.y,
                        }
                        .intersect(current_p, next_p),
                    ));
                }
                if next_i.y < current_i.y {
                    intersections.push((
                        Index::new(0, -1),
                        Line::Horizontal {
                            y: current_box.min.y,
                        }
                        .intersect(current_p, next_p),
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
