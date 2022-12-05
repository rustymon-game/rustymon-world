//! Ast and grammar for the custom config language
use std::collections::{HashMap, HashSet};
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
#[derive(Debug)]
pub struct Ast<T> {
    /// The `[Areas]` block
    pub areas: Vec<Branch<T>>,

    /// The `[Nodes]` block
    pub nodes: Vec<Branch<T>>,

    /// The `[Ways]` block
    pub ways: Vec<Branch<T>>,
}

/// A matching branch maps a condition to a result.
#[derive(Debug)]
pub struct Branch<T> {
    /// The branch's result
    pub id: usize,

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
            Ok(Ast {
                areas: Vec::new(),
                nodes: Vec::new(),
                ways: Vec::new(),
            })
        }
    }

    fn handle_file(&mut self, file: Pair<'i, Rule>) -> ParserResult<Ast<T>> {
        let mut area_aliases: HashMap<&'i str, usize> = HashMap::new();
        let mut node_aliases: HashMap<&'i str, usize> = HashMap::new();
        let mut way_aliases: HashMap<&'i str, usize> = HashMap::new();
        Ok(match file.as_rule() {
            Rule::file => {
                let mut areas = None;
                let mut nodes = None;
                let mut ways = None;
                for block in file.into_inner() {
                    let rule = block.as_rule();
                    match rule {
                        Rule::block => (),
                        Rule::EOI => continue,
                        _ => return invalid_rule(rule, [Rule::block, Rule::EOI]),
                    }

                    let (title, statements) = block.head_tail().ok_or(missing_child(rule))?;
                    let (block, aliases) = match title.as_rule() {
                        Rule::areas => (&mut areas, &mut area_aliases),
                        Rule::nodes => (&mut nodes, &mut node_aliases),
                        Rule::ways => (&mut ways, &mut way_aliases),
                        invalid => {
                            return Err(ParserError::InvalidRule(
                                invalid,
                                vec![Rule::areas, Rule::nodes, Rule::ways],
                            ))
                        }
                    };
                    if block.is_none() {
                        let mut branches = Vec::new();
                        for rule in statements {
                            self.handle_statement(rule, &mut branches, aliases)?;
                        }
                        *block = Some(branches)
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
            i => return invalid_rule(i, [Rule::file]),
        })
    }

    fn handle_statement(
        &mut self,
        stmnt: Pair<'i, Rule>,
        branches: &mut Vec<Branch<T>>,
        aliases: &mut HashMap<&'i str, usize>,
    ) -> ParserResult<()> {
        let rule = stmnt.as_rule();
        match rule {
            Rule::statement => {
                self.handle_statement(stmnt.child().ok_or(missing_child(rule))?, branches, aliases)?
            }
            Rule::alias => {
                let [identifier, number] = stmnt.children().ok_or(missing_child(rule))?;
                aliases.insert(identifier.as_str(), number.as_str().parse().unwrap());
            }
            Rule::branch => {
                let [result, expr] = stmnt.children().ok_or(missing_child(rule))?;
                let id = match result.as_rule() {
                    Rule::number => result.as_str().parse().unwrap(),
                    Rule::identifier => *aliases
                        .get(result.as_str())
                        .ok_or(ParserError::UnknownAlias(result.as_str().to_string()))?,
                    _ => return invalid_rule(result.as_rule(), [Rule::number, Rule::identifier]),
                };
                let branch = Branch {
                    id,
                    expr: self.handle_expr(expr)?,
                };
                branches.push(branch);
            }
            _ => return invalid_rule(rule, [Rule::statement, Rule::alias, Rule::branch]),
        }
        Ok(())
    }

    fn handle_expr(&mut self, expr: Pair<'i, Rule>) -> ParserResult<Expr<T>> {
        let rule = expr.as_rule();
        Ok(match rule {
            Rule::expr => self.handle_expr(expr.child().ok_or(missing_child(rule))?)?,
            Rule::lookup => {
                Expr::Lookup(self.handle_lookup(expr.child().ok_or(missing_child(rule))?)?)
            }
            Rule::not => Expr::Not(Box::new(
                self.handle_expr(expr.child().ok_or(missing_child(rule))?)?,
            )),
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
            _ => {
                return invalid_rule(
                    rule,
                    [Rule::expr, Rule::lookup, Rule::not, Rule::or, Rule::and],
                )
            }
        })
    }

    fn handle_lookup(&mut self, lookup: Pair<'i, Rule>) -> ParserResult<Lookup<T>> {
        let rule = lookup.as_rule();
        Ok(match lookup.as_rule() {
            Rule::lookup => self.handle_lookup(lookup.child().ok_or(missing_child(rule))?)?,
            Rule::any => Lookup::Any {
                key: self.handle_string(lookup.child().ok_or(missing_child(rule))?)?,
            },
            Rule::single => {
                let [key, value] = lookup.children().ok_or(missing_child(rule))?;
                Lookup::Single {
                    key: self.handle_string(key)?,
                    value: self.handle_string(value)?,
                }
            }
            Rule::list => {
                let (key, values) = lookup.head_tail().ok_or(missing_child(rule))?;
                Lookup::List {
                    key: self.handle_string(key)?,
                    values: values
                        .map(|rule| self.handle_string(rule))
                        .collect::<ParserResult<_>>()?,
                }
            }
            _ => return invalid_rule(rule, [Rule::lookup, Rule::any, Rule::single, Rule::list]),
        })
    }

    fn handle_string(&mut self, string: Pair<'i, Rule>) -> ParserResult<T> {
        let inner = match string.as_rule() {
            Rule::string => string.child().ok_or(missing_child(Rule::string))?,
            Rule::inner => string,
            _ => return invalid_rule(string.as_rule(), [Rule::string, Rule::inner]),
        };
        Ok((self.convert_string)(inner.as_str()))
    }
}

#[derive(Debug)]
pub enum ParserError {
    /// A syntax error found by pest's parser
    SyntaxError(pest::error::Error<Rule>),

    /// A block appeared twice
    DuplicateBlocks,

    /// An alias was used before it was declared
    UnknownAlias(String),

    /// A rule is missing a specific child, whose existence should be guaranteed by the grammar
    ///
    /// This is to be treated as a mistake in this library.
    MissingChild(Rule),

    /// A `ConfigParser::handle_*` function got a rule it can't handle
    ///
    /// This is to be treated as a mistake in this library.
    InvalidRule(Rule, Vec<Rule>),
}
impl From<pest::error::Error<Rule>> for ParserError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        Self::SyntaxError(error)
    }
}
fn invalid_rule<T, const N: usize>(got: Rule, expected: [Rule; N]) -> ParserResult<T> {
    Err(ParserError::InvalidRule(got, expected.to_vec()))
}
fn missing_child(parent: Rule) -> ParserError {
    ParserError::MissingChild(parent)
}
impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::SyntaxError(err) => err.fmt(f),
            ParserError::DuplicateBlocks => write!(f, "Some blocks where specified twice"),
            ParserError::MissingChild(parent) => {
                write!(
                    f,
                    "This error should never happen! Please tell the maintainer!\n"
                )?;
                write!(f, "Parent: {:?}", parent)
            }
            ParserError::InvalidRule(got, exp) => {
                write!(
                    f,
                    "This error should never happen! Please tell the maintainer!\n"
                )?;
                write!(f, "Got: {:?}\n", got)?;
                if exp.len() == 1 {
                    write!(f, "Expected: {:?}\n", exp[1])
                } else {
                    write!(f, "Expected one of:\n")?;
                    for rule in exp {
                        write!(f, "- {:?}\n", rule)?;
                    }
                    Ok(())
                }
            }
            ParserError::UnknownAlias(alias) => {
                write!(f, "The alias \"{alias}\" was used before its declaration")
            }
        }
    }
}

type ParserResult<T> = Result<T, ParserError>;
