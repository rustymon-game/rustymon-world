use crate::features::config::{Expr, Lookup};
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub enum SimpleExpr<T: Copy> {
    /// This value is a temporary replacement and should never appear in the tree outside of its methods.
    Temp(Private),

    Not(Box<SimpleExpr<T>>),
    And(Vec<SimpleExpr<T>>),
    Or(Vec<SimpleExpr<T>>),
    Terminal(T),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Private(PhantomData<()>);
impl Private {
    const fn new() -> Self {
        Self(PhantomData)
    }
}
impl std::fmt::Debug for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ERROR").finish()
    }
}

impl<T: Copy> SimpleExpr<T> {
    /// Change the expression node in place using a conversion function.
    fn change_inplace(&mut self, mut func: impl FnMut(Self) -> Self) {
        // It is safe to take the value from safe and leave an invalid state,
        // because it will be reassigned an new valid one immediately.
        *self = func(std::mem::replace(self, Self::Temp(Private::new())));
    }

    // The common flatten logic shared by Self::And and Self::Or
    fn _flatten(vec: &mut Vec<Self>, is_variant: impl Fn(&Self) -> bool) -> bool {
        let mut changed = false;
        let mut index = 0;
        let mut unchecked = vec.len();
        while index < unchecked {
            if is_variant(&vec[index]) {
                match vec.swap_remove(index) {
                    Self::And(inner) | Self::Or(inner) => vec.extend(inner.into_iter()),
                    _ => unreachable!(),
                }
                changed = true;

                unchecked -= 1;
            } else {
                index += 1;
            }
        }
        changed
    }

    fn flatten(&mut self) -> bool {
        match self {
            Self::Temp(_) => unreachable!(),
            Self::Not(inner) => match inner.as_mut() {
                Self::Not(new_self) => {
                    // `new_self` is contained in the current self which will be overwritten
                    // dropping the two boxes.
                    // Therefore, it is ok to leave an invalid value in the second box.
                    *self = std::mem::replace(new_self.as_mut(), Self::Temp(Private::new()));
                    true
                }
                _ => false,
            },
            Self::And(vec) => Self::_flatten(vec, |expr| matches!(expr, Self::And(_))),
            Self::Or(vec) => Self::_flatten(vec, |expr| matches!(expr, Self::Or(_))),
            Self::Terminal(_) => false,
        }
    }

    fn simplify(&mut self) -> bool {
        let mut changed = false;
        match self {
            Self::Temp(_) => unreachable!(),
            SimpleExpr::Not(inner) => {
                match inner.as_mut() {
                    Self::Temp(_) => unreachable!(),

                    // Flattening will be done further below
                    SimpleExpr::Not(inner) => changed |= inner.simplify(),

                    SimpleExpr::And(vec) => {
                        for expr in vec.iter_mut() {
                            expr.change_inplace(|expr| SimpleExpr::Not(Box::new(expr)));
                            expr.simplify();
                        }
                        *self = SimpleExpr::Or(std::mem::take(vec));
                        changed = true;
                    }

                    SimpleExpr::Or(vec) => {
                        for expr in vec.iter_mut() {
                            expr.change_inplace(|expr| SimpleExpr::Not(Box::new(expr)));
                            expr.simplify();
                        }
                        *self = SimpleExpr::And(std::mem::take(vec));
                        changed = true;
                    }

                    SimpleExpr::Terminal(_) => (),
                }
            }
            SimpleExpr::And(vec) => {
                let mut index = None;
                for (i, e) in vec.iter().enumerate() {
                    if matches!(e, SimpleExpr::Or(_)) {
                        index = Some(i);
                    }
                }
                // Found an `Or` inside the `And`
                if let Some(index) = index {
                    let inner_or = match vec.remove(index) {
                        SimpleExpr::Or(inner_or) => inner_or,
                        _ => unreachable!(),
                    };
                    let mut outer_vec = Vec::with_capacity(inner_or.len());
                    for expr in inner_or {
                        let mut inner_vec = vec.clone();
                        inner_vec.reserve(1);
                        inner_vec.push(expr);
                        let mut inner_and = SimpleExpr::And(inner_vec);
                        inner_and.simplify();
                        outer_vec.push(inner_and);
                    }
                    *self = SimpleExpr::Or(outer_vec);
                    changed = true;
                } else {
                    for elem in vec {
                        changed |= elem.simplify();
                    }
                }
            }
            SimpleExpr::Or(vec) => {
                for elem in vec {
                    changed |= elem.simplify();
                }
            }
            SimpleExpr::Terminal(_) => (),
        };
        self.flatten() || changed
    }
}

impl<T: Copy> SimpleExpr<(T, Option<T>)> {
    fn from_config(expr: &Expr<T>) -> Self {
        match expr {
            Expr::Not(inner) => Self::Not(Box::new(Self::from_config(inner))),
            Expr::And(vec) => Self::And(vec.iter().map(Self::from_config).collect()),
            Expr::Or(vec) => Self::Or(vec.iter().map(Self::from_config).collect()),
            Expr::Lookup(Lookup::Any { key }) => Self::Terminal((*key, None)),
            Expr::Lookup(Lookup::Single { key, value }) => Self::Terminal((*key, Some(*value))),
            Expr::Lookup(Lookup::List { key, values }) => Self::Or(
                values
                    .into_iter()
                    .map(|value| Self::Terminal((*key, Some(*value))))
                    .collect(),
            ),
        }
    }
}

pub fn simplify<T: Copy>(expr: &Expr<T>) -> SimpleExpr<(T, Option<T>)> {
    let mut expr = SimpleExpr::from_config(expr);

    while expr.simplify() {}

    expr
}

#[cfg(test)]
mod test {
    use crate::features::simplify::SimpleExpr;

    type Expr = SimpleExpr<usize>;

    fn not(expr: Expr) -> Expr {
        SimpleExpr::Not(Box::new(expr))
    }

    fn term(index: usize) -> Expr {
        SimpleExpr::Terminal(index)
    }

    fn or<const N: usize>(arr: [Expr; N]) -> Expr {
        SimpleExpr::Or(arr.to_vec())
    }

    fn and<const N: usize>(arr: [Expr; N]) -> Expr {
        SimpleExpr::And(arr.to_vec())
    }

    fn steps(mut expr: Expr) -> Expr {
        while expr.simplify() {}
        expr
    }

    #[test]
    fn flatten_not() {
        let mut expr = not(not(not(term(1))));
        assert!(expr.flatten());
        assert_eq!(expr, not(term(1)));
        assert!(!expr.flatten());
        assert_eq!(expr, not(term(1)));
    }

    #[test]
    fn flatten_and() {
        let mut expr = and([
            and([term(1), term(2)]),
            term(3),
            and([and([term(4), term(5)]), term(6)]),
        ]);
        assert!(expr.flatten());
        assert_eq!(
            expr,
            and([term(2), term(3), term(1), and([term(4), term(5)]), term(6)])
        );
        assert!(expr.flatten());
        assert_eq!(
            expr,
            and([term(2), term(3), term(1), term(6), term(4), term(5)])
        );
        assert!(!expr.flatten());
        assert_eq!(
            expr,
            and([term(2), term(3), term(1), term(6), term(4), term(5)])
        );
    }

    #[test]
    fn flatten_or() {
        let mut expr = or([
            or([term(1), term(2)]),
            term(3),
            or([or([term(4), term(5)]), term(6)]),
        ]);
        assert!(expr.flatten());
        assert_eq!(
            expr,
            or([term(2), term(3), term(1), or([term(4), term(5)]), term(6),]),
        );
        assert!(expr.flatten());
        assert_eq!(
            expr,
            or([term(2), term(3), term(1), term(6), term(4), term(5),]),
        );
        assert!(!expr.flatten());
        assert_eq!(
            expr,
            or([term(2), term(3), term(1), term(6), term(4), term(5),]),
        );
    }

    #[test]
    fn or_in_and() {
        assert_eq!(
            steps(and([
                or([term(1), term(2)]),
                or([term(3), term(4)]),
                or([term(5), term(6)])
            ]),),
            steps(or([
                and([term(5), term(4), term(2)]),
                and([term(5), term(3), term(2)]),
                and([term(5), term(3), term(1)]),
                and([term(5), term(4), term(1)]),
                and([term(6), term(3), term(2)]),
                and([term(6), term(3), term(1)]),
                and([term(6), term(4), term(1)]),
                and([term(6), term(4), term(2)]),
            ])),
        );
    }

    #[test]
    fn or_in_not() {
        assert_eq!(
            steps(not(or([term(1), term(2)]))),
            and([not(term(1)), not(term(2))]),
        );
    }

    #[test]
    fn and_in_not() {
        assert_eq!(
            steps(not(and([term(1), term(2)]))),
            or([not(term(1)), not(term(2))]),
        );
    }
}
