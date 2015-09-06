//! Keeps `WHERE` related stuff.

use common::ToSQL;

#[derive(Clone, PartialEq, Eq)]
pub enum Operator {
    And,
    Or
}

impl Operator {
    pub fn to_sql(&self) -> &str {
        match *self {
            Operator::And => "AND",
            Operator::Or => "OR"
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum WhereType<'a> {
    Simple(&'a str),
    Extended(Where<'a>),
}


/// Represents `WHERE` clause which is widely used in different queries.
#[derive(Clone, PartialEq, Eq)]
pub struct Where<'a> {
    /// Operator which will be used to join expressions
    pub operator: Operator,
    expressions: Vec<WhereType<'a>>,
}

impl<'a> Where<'a> {
    /// Method to start with.
    pub fn new(operator: Operator) -> Self {
        Where {
            operator: operator,
            expressions: vec![]
        }
    }
    
    /// Just an alias for `new` with pre-defined `AND` operator.
    pub fn with_and() -> Self {
        Self::new(Operator::And)
    }

    /// Just an alias for `new` with pre-defined `OR` operator.
    pub fn with_or() -> Self {
        Self::new(Operator::Or)
    }

    /// Specifies clause. Can receive either `&str` or `Where`.
    pub fn expr<T: IntoWhereType<'a>>(mut self, expression: T) -> Self {
        self.expressions.push(expression.into_where_type());
        self
    }
}

pub trait IntoWhereType<'a> {
    fn into_where_type(self) -> WhereType<'a>;
}

impl<'a> IntoWhereType<'a> for &'a str {
    fn into_where_type(self) -> WhereType<'a> {
        WhereType::Simple(self)
    }
}

impl<'a> IntoWhereType<'a> for Where<'a> {
    fn into_where_type(self) -> WhereType<'a> {
        WhereType::Extended(self)
    }
}

impl<'a> ToSQL for WhereType<'a> {
    fn to_sql(&self) -> String {
        match *self {
            WhereType::Simple(clause) => clause.to_string(),
            WhereType::Extended(ref clause) => clause.to_sql()
        }
    }
}

impl<'a> ToSQL for &'a str {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl<'a> ToSQL for Where<'a> {
    fn to_sql(&self) -> String {
        let operator = format!(" {} ", self.operator.to_sql());
        let mut rv = String::new();
        rv.push('(');
        rv.push_str(&self.expressions.iter()
                    .map(|x| x.to_sql())
                    .collect::<Vec<_>>()
                    .join(&operator));

        rv.push(')');
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{Operator, Where};
    use common::ToSQL;

    #[test]
    fn test_operator() {
        let and = Operator::And;
        let or = Operator::Or;

        assert_eq!(and.to_sql(), "AND");
        assert_eq!(or.to_sql(), "OR");
    }

    #[test]
    fn test_alone_where() {
        let foo = Where::new(Operator::And).expr("foo == bar").expr("fizz == bazz");
        assert_eq!(foo.to_sql(), "(foo == bar AND fizz == bazz)".to_string())
    }

    #[test]
    fn test_nested_where_clauses() {
        let clause = Where::with_or()
            .expr(Where::with_and().expr("foo == bar").expr("fizz == bazz"))
            .expr(Where::with_and().expr("a == b").expr("c == d"));

        let test_sql_string = {
            "((foo == bar AND fizz == bazz) OR \
            (a == b AND c == d))".to_string()
        };
        assert_eq!(clause.to_sql(), test_sql_string);
    }

    #[test]
    fn test_really_nested_where_clauses() {
        let foo = Where::with_and().expr("foo == bar").expr("fizz == bazz");
        let bar = Where::with_and().expr("a == b").expr("c == d");
        let bazz1 = Where::with_or().expr(foo.clone()).expr(bar.clone());
        let bazz2 = Where::with_or().expr(bar.clone()).expr(foo.clone());
        let fizz = Where::with_and().expr(bazz1).expr(bazz2);

        let test_sql_string = {
            "(((foo == bar AND fizz == bazz) OR \
            (a == b AND c == d)) AND \
            ((a == b AND c == d) OR \
            (foo == bar AND fizz == bazz)))".to_string()
        };
        assert_eq!(fizz.to_sql(), test_sql_string);
    }
}
