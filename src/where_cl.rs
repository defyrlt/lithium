//! Keeps `WHERE` related stuff.

use common::{ToSQL, Numeric, Subquery};

#[derive(Clone)]
pub enum Operator {
    And,
    Or
}

impl Operator {
    pub fn to_sql(&self) -> &'static str {
        match *self {
            Operator::And => "AND",
            Operator::Or => "OR"
        }
    }
}

pub trait WhereType<'a>: ToSQL + CloneToTrait<'a> {}
impl<'a> WhereType<'a> for &'a str {}
impl<'a, T: Numeric + ToSQL + Clone> WhereType<'a> for T {}
impl<'a> WhereType<'a> for Where<'a> {}
// TODO: find a nice way to do it without cloning
// impl<'a> WhereType for &'a Subquery<'a> {}

pub trait CloneToTrait<'a>: 'a {
    fn clone_to_trait(&self) -> Box<WhereType<'a>>;
}

impl<'a, T: Clone + WhereType<'a>> CloneToTrait<'a> for T {
    fn clone_to_trait(&self) -> Box<WhereType<'a>> {
        Box::new(self.clone()) 
    }
}

impl<'a> Clone for Box<WhereType<'a>> {
    fn clone(&self) -> Box<WhereType<'a>> {
        self.clone_to_trait()
    }
}

/// Represents `WHERE` clause which is widely used in different queries.
#[derive(Clone)]
pub struct Where<'a> {
    /// Operator which will be used to join filters
    pub operator: Operator,
    filters: Vec<Box<WhereType<'a> + 'a>>,
}

impl<'a> Where<'a> {
    /// Method to start with.
    pub fn new(operator: Operator) -> Self {
        Where {
            operator: operator,
            filters: vec![]
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

    pub fn filter<T: WhereType<'a> + 'a>(mut self, raw: T) -> Self {
        self.filters.push(Box::new(raw));
        self
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
        rv.push_str(&self.filters.iter()
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
        let foo = Where::new(Operator::And).filter("foo = bar").filter("fizz = bazz");
        assert_eq!(foo.to_sql(), "(foo = bar AND fizz = bazz)".to_string());
    }

    #[test]
    fn test_nested_where_clauses() {
        let clause = Where::with_or()
            .filter(Where::with_and().filter("foo != bar").filter("fizz = bazz"))
            .filter(Where::with_and().filter("a = b").filter("c = d"));

        let test_sql_string = {
            "((foo != bar AND fizz = bazz) OR \
            (a = b AND c = d))".to_string()
        };
        assert_eq!(clause.to_sql(), test_sql_string);
    }

    #[test]
    fn test_really_nested_where_clauses() {
        let foo = Where::with_and().filter("foo = bar").filter("fizz = 2");
        let bar = Where::with_and().filter("a = b").filter("c = d");

        let bazz1 = Where::with_or().filter(foo.clone()).filter(bar.clone());
        let bazz2 = Where::with_or().filter(bar.clone()).filter(foo.clone());
        let fizz = Where::with_and().filter(bazz1).filter(bazz2);

        let test_sql_string = {
            "(((foo = bar AND fizz = 2) OR \
            (a = b AND c = d)) AND \
            ((a = b AND c = d) OR \
            (foo = bar AND fizz = 2)))".to_string()
        };
        assert_eq!(fizz.to_sql(), test_sql_string);
    }
}
