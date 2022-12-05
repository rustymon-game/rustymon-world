//! Ast and grammar for the custom config language
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

use pest::iterators::Pair;
use pest::Parser;

use super::pest_ext::PairsExt;

/// The config's grammar defined using [pest](https://pest.rs/)
#[derive(pest_derive::Parser)]
#[grammar = "features/config.pest"]
struct Grammar;

/// The AST representing a config file.
#[derive(Debug, Default)]
pub struct Ast<T> {
    /// The `[Areas]` block
    areas: Vec<Branch<T>>,

    /// The `[Nodes]` block
    nodes: Vec<Branch<T>>,

    /// The `[Ways]` block
    ways: Vec<Branch<T>>,
}

/// A matching branch maps a condition to a result.
#[derive(Debug)]
pub struct Branch<T> {
    /// The branch's result
    pub id: u32,

    /// The branch's condition
    pub expr: Expr<T>,
}

/// A condition is a boolean expression
#[derive(Debug)]
pub enum Expr<T> {
    /// Invert an expression
    Not(Box<Expr<T>>),

    /// Combine expressions using logical and
    And(Vec<Expr<T>>),

    /// Combine expressions using logical or
    Or(Vec<Expr<T>>),

    /// The primitive all expressions are build from
    Lookup(Lookup<T>),
}

/// A lookup operation which checks for a specific tag in the tag list.
#[derive(Debug)]
pub enum Lookup<T> {
    /// Check if the tag is present, ignoring its value
    Any { key: T },

    /// Check if the tag has a concrete value
    Single { key: T, value: T },

    /// Check if the tag's value is part of a list
    List { key: T, values: HashSet<T> },
}

/// Parser to produce an [Ast] from a config string
pub struct ConfigParser<'i, T, F>
where
    F: FnMut(&'i str) -> T,
{
    convert_string: F,
    phantom: PhantomData<(&'i str, T)>,
}

impl<'i, T, F> ConfigParser<'i, T, F>
where
    F: FnMut(&'i str) -> T,
{
    /// Wrap a closure to convert the lookup strings into a custom value
    pub fn new(convert_string: F) -> Self {
        Self {
            convert_string,
            phantom: PhantomData,
        }
    }
}
impl<'i> ConfigParser<'i, &'i str, fn(&'i str) -> &'i str> {
    /// Get a parser who builds the ast by borrowing the input string
    pub const fn borrowing() -> Self {
        Self {
            convert_string: std::convert::identity,
            phantom: PhantomData,
        }
    }
}
impl<'i> ConfigParser<'i, String, fn(&'i str) -> String> {
    /// Get a parser who builds the ast by cloning the input string
    pub const fn owning() -> Self {
        Self {
            convert_string: str::to_string,
            phantom: PhantomData,
        }
    }
}

impl<'i, T, F> ConfigParser<'i, T, F>
where
    T: Eq + Hash, // requirements for hashset
    F: FnMut(&'i str) -> T,
{
    pub fn parse_file(mut self, expr: &'i str) -> ParserResult<Ast<T>> {
        let mut matches = Grammar::parse(Rule::file, expr)?;
        if let Some(file) = matches.next() {
            self.handle_file(file)
        } else {
            Ok(Ast::default())
        }
    }

    fn handle_file(&mut self, file: Pair<'i, Rule>) -> ParserResult<Ast<T>> {
        Ok(match file.as_rule() {
            Rule::file => {
                let mut areas = None;
                let mut nodes = None;
                let mut ways = None;
                for block in file.into_inner() {
                    let (title, statements) = block.head_tail().ok_or(MISSING)?;
                    let block = match title.as_rule() {
                        Rule::areas => &mut areas,
                        Rule::nodes => &mut nodes,
                        Rule::ways => &mut ways,
                        _ => return Err(ParserError::InvalidRule),
                    };
                    if block.is_none() {
                        *block = Some(
                            statements
                                .map(|rule| self.handle_statement(rule))
                                .collect::<ParserResult<_>>()?,
                        );
                    } else {
                        return Err(ParserError::DuplicateBlocks);
                    }
                }
                Ast {
                    areas: areas.unwrap_or_default(),
                    nodes: nodes.unwrap_or_default(),
                    ways: ways.unwrap_or_default(),
                }
            }
            _ => return Err(ParserError::InvalidRule),
        })
    }

    fn handle_statement(&mut self, stmnt: Pair<'i, Rule>) -> ParserResult<Branch<T>> {
        Ok(match stmnt.as_rule() {
            Rule::statement => {
                let [id, expr] = stmnt.children().ok_or(MISSING)?;
                Branch {
                    id: id.as_str().parse().unwrap(),
                    expr: self.handle_expr(expr)?,
                }
            }
            _ => return Err(ParserError::InvalidRule),
        })
    }

    fn handle_expr(&mut self, expr: Pair<'i, Rule>) -> ParserResult<Expr<T>> {
        Ok(match expr.as_rule() {
            Rule::expr => self.handle_expr(expr.child().ok_or(MISSING)?)?,
            Rule::lookup => Expr::Lookup(self.handle_lookup(expr.child().ok_or(MISSING)?)?),
            Rule::not => Expr::Not(Box::new(self.handle_expr(expr.child().ok_or(MISSING)?)?)),
            Rule::or => Expr::Or(
                expr.into_inner()
                    .map(|rule| self.handle_expr(rule))
                    .collect::<ParserResult<_>>()?,
            ),
            Rule::and => Expr::And(
                expr.into_inner()
                    .map(|rule| self.handle_expr(rule))
                    .collect::<ParserResult<_>>()?,
            ),
            _ => return Err(ParserError::InvalidRule),
        })
    }

    fn handle_lookup(&mut self, lookup: Pair<'i, Rule>) -> ParserResult<Lookup<T>> {
        Ok(match lookup.as_rule() {
            Rule::lookup => self.handle_lookup(lookup.child().ok_or(MISSING)?)?,
            Rule::any => Lookup::Any {
                key: self.handle_string(lookup.child().ok_or(MISSING)?)?,
            },
            Rule::single => {
                let [key, value] = lookup.children().ok_or(MISSING)?;
                Lookup::Single {
                    key: self.handle_string(key)?,
                    value: self.handle_string(value)?,
                }
            }
            Rule::list => {
                let (key, values) = lookup.head_tail().ok_or(MISSING)?;
                Lookup::List {
                    key: self.handle_string(key)?,
                    values: values
                        .map(|rule| self.handle_string(rule))
                        .collect::<ParserResult<_>>()?,
                }
            }
            _ => return Err(ParserError::InvalidRule),
        })
    }

    fn handle_string(&mut self, string: Pair<'i, Rule>) -> ParserResult<T> {
        Ok(match string.as_rule() {
            Rule::string | Rule::inner => {
                let string = string.as_str();
                (self.convert_string)(string)
            }
            _ => return Err(ParserError::InvalidRule),
        })
    }
}

#[derive(Debug)]
pub enum ParserError {
    /// A syntax error found by pest's parser
    SyntaxError(pest::error::Error<Rule>),

    /// A block appeared twice
    DuplicateBlocks,

    /// A rule is missing a specific child, whose existence should be guaranteed by the grammar
    ///
    /// This is to be treated as a mistake in this library.
    MissingChild,

    /// A `ConfigParser::handle_*` function got a rule it can't handle
    ///
    /// This is to be treated as a mistake in this library.
    InvalidRule,
}
const MISSING: ParserError = ParserError::MissingChild;
impl From<pest::error::Error<Rule>> for ParserError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        Self::SyntaxError(error)
    }
}

type ParserResult<T> = Result<T, ParserError>;
