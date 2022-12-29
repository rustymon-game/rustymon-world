use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::features::config::{Ast, Branch, Expr, Lookup};
use crate::features::{FeatureParser, Tags};

impl FeatureParser for Ast<&str> {
    type Feature = usize;

    fn area<'t>(&self, area: impl Tags<'t>) -> Option<Self::Feature> {
        Self::parse_tags(&self.areas, area)
    }

    fn node<'t>(&self, node: impl Tags<'t>) -> Option<Self::Feature> {
        Self::parse_tags(&self.nodes, node)
    }

    fn way<'t>(&self, way: impl Tags<'t>) -> Option<Self::Feature> {
        Self::parse_tags(&self.ways, way)
    }
}

impl<'i> Ast<&'i str> {
    fn parse_tags<'t, 'm>(statements: &[Branch<&'i str>], tags: impl Tags<'t>) -> Option<usize>
    where
        't: 'm,
        'i: 'm,
    {
        let tags: HashMap<&'m str, &'m str> =
            HashMap::from_iter(tags.into_iter().map(|(k, v)| (k.borrow(), v.borrow())));
        for statement in statements {
            if eval_expr(&statement.expr, &tags) {
                return Some(statement.id);
            }
        }
        None
    }
}

pub(crate) fn eval_expr<E, T>(expr: &Expr<E>, tags: &HashMap<T, T>) -> bool
where
    E: Eq + Hash, // `expr`'s tree contains a HashSet<E>
    E: Borrow<T>, // Expr::Lookup contains `key` of type E which is used to index `tags`
    T: Eq + Hash, // `tags` is a HashMap<T, T>
{
    match expr {
        Expr::Not(expr) => !eval_expr(expr, tags),
        Expr::And(list) => list.iter().all(|expr| eval_expr(expr, tags)),
        Expr::Or(list) => list.iter().any(|expr| eval_expr(expr, tags)),
        Expr::Lookup(lookup) => {
            let key = match lookup {
                Lookup::Any { key } => key,
                Lookup::Single { key, .. } => key,
                Lookup::List { key, .. } => key,
            }
            .borrow();
            let Some(tag_value) = tags.get(key) else {return false;};
            match lookup {
                Lookup::Any { .. } => true,
                Lookup::Single {
                    value: exp_value, ..
                } => exp_value.borrow() == tag_value,
                Lookup::List {
                    values: pos_values, ..
                } => pos_values.contains(tag_value),
            }
        }
    }
}
