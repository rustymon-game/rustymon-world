use crate::features::simplify::SimpleExpr;

pub type SortedVec<T> = Vec<T>;

pub struct Branch<T: Copy>(Vec<SortedVec<Atom<T>>>);

type Expr<T> = SimpleExpr<(T, Option<T>)>;
impl<T: Copy + Ord> Branch<T> {
    fn convert_atom(not: &Expr<T>) -> Result<Atom<T>, ()> {
        match not {
            Expr::Not(inner) => match inner.as_ref() {
                Expr::Terminal((key, value)) => Ok(Atom {
                    key: *key,
                    value: *value,
                    not: true,
                }),
                _ => Err(()),
            },
            Expr::Terminal((key, value)) => Ok(Atom {
                key: *key,
                value: *value,
                not: false,
            }),
            _ => Err(()),
        }
    }

    fn convert_and(expr: &Expr<T>) -> Result<Vec<Atom<T>>, ()> {
        match expr {
            Expr::Not(_) | Expr::Terminal(_) => Ok(vec![Self::convert_atom(expr)?]),
            Expr::And(vec) => vec.iter().map(Self::convert_atom).collect::<Result<_, _>>(),
            _ => Err(()),
        }
    }

    pub fn from_simplified(expr: &Expr<T>) -> Result<Self, ()> {
        let mut outer_vec = match expr {
            Expr::Not(_) | Expr::Terminal(_) => Ok(vec![vec![Self::convert_atom(expr)?]]),
            Expr::And(_) => Ok(vec![Self::convert_and(expr)?]),
            Expr::Or(vec) => vec.iter().map(Self::convert_and).collect::<Result<_, _>>(),
            _ => Err(()),
        }?;

        for and in outer_vec.iter_mut() {
            and.sort_unstable_by_key(Atom::sort_key);
        }

        Ok(Self(outer_vec))
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Atom<T: Copy> {
    key: T,
    not: bool,
    value: Option<T>,
}
impl<T: Copy> Atom<T> {
    /// Pass this function to `sort_by_key` or similar methods when sorting a `&mut [Atom<T>]`
    pub fn sort_key(&self) -> T {
        self.key
    }
}
