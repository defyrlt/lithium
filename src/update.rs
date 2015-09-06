//! Keeps `UPDATE` related stuff.

use common::{ToSQL, Pusheable, AsStr};
use where_cl::{WhereType, IntoWhereType};

// TODO: make it pretty
const RETURNING: &'static str = " RETURNING ";

#[derive(Clone, PartialEq, Eq)]
pub enum FromType<'a> {
    Empty,
    Specified(&'a str)
}

#[derive(Clone, PartialEq, Eq)]
pub enum Returning<'a> {
    Empty,
    All,
    Specified(Vec<&'a str>)
}

/// Represents `UPDATE` query
#[derive(Clone, PartialEq, Eq)]
pub struct Update<'a> {
    table: &'a str,
    expressions: Vec<&'a str>,
    from: FromType<'a>,
    where_cl: Vec<WhereType<'a>>,
    returning: Returning<'a>
}

impl<'a> Update<'a> {
    /// Method to start with.
    pub fn new(table: &'a str) -> Self {
        Update {
            table: table,
            expressions: vec![],
            from: FromType::Empty,
            where_cl: vec![],
            returning: Returning::Empty
        }
    }

    /// Specifies update expressions.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Update;
    /// let query = Update::new("foo").set("a = 1").set(&["b = 2", "c = 3"]);
    /// let expected = "UPDATE foo SET a = 1, b = 2, c = 3".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    pub fn set<T: Pusheable<'a>>(mut self, expressions: T) -> Self {
        expressions.push_to(&mut self.expressions);
        self
    }

    /// Specifies `FROM` clause. Can take either `&str` or `&Subquery`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithium::Update;
    /// let query = Update::new("foo").set("a = blah.a").from("blah");
    /// let expected = "UPDATE foo SET a = blah.a FROM blah".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    ///
    /// ```
    /// use lithium::{Select, Update};
    /// let subquery = Select::from("foo").columns(&["a", "b"]).as_subquery().with_alias("foo");
    /// let update = Update::new("bar").set(&["a = foo.a", "b = foo.b"]).from(&subquery);
    /// let expected = "UPDATE bar SET a = foo.a, b = foo.b FROM (SELECT a, b FROM foo) AS foo".to_string();
    /// assert_eq!(update.to_sql(), expected);
    /// ```
    pub fn from<T: AsStr<'a>>(mut self, table: T) -> Self {
        self.from = FromType::Specified(table.as_str());
        self
    }

    /// Removes `FROM` clause.
    pub fn remove_from(mut self) -> Self {
        self.from = FromType::Empty;
        self
    }

    /// Specifies `WHERE` clause. Can take either `&str` or `Where`.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::{Update, Where};
    /// let where_cl = Where::with_or().expr("a > 2").expr("b < 3");
    /// let update = Update::new("foo").set("a = 2").filter(where_cl).filter("c > 4");
    /// let expected = "UPDATE foo SET a = 2 WHERE (a > 2 OR b < 3) AND c > 4".to_string();
    /// assert_eq!(update.to_sql(), expected);
    /// ```
    pub fn filter<T: IntoWhereType<'a>>(mut self, expr: T) -> Self {
        self.where_cl.push(expr.into_where_type());
        self
    }

    /// Specifies `RETURNING` clause. Will result in `UPDATE ... RETURNING *`
    pub fn returning_all(mut self) -> Self {
        self.returning = Returning::All;
        self
    }

    /// Specifies `RETURNING` clause.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Update;
    /// let query = Update::new("test_table").set("a = 2").returning("a").returning(&["b", "c"]);
    /// let expected = "UPDATE test_table SET a = 2 RETURNING a, b, c".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    pub fn returning<T: Pusheable<'a>>(mut self, input_expressions: T) -> Self {
        match self.returning {
            Returning::Empty | Returning::All => {
                let mut expressions = vec![];
                input_expressions.push_to(&mut expressions);
                self.returning = Returning::Specified(expressions);
            },
            Returning::Specified(ref mut expressions) => input_expressions.push_to(expressions)
        }
        self
    }

    /// Remooves `RETURNING` clause
    pub fn empty_returning(mut self) -> Self {
        self.returning = Returning::Empty;
        self
    }

    /// Generates SQL.
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("UPDATE");
        rv.push(' ');
        rv.push_str(self.table);
        rv.push(' ');
        rv.push_str("SET");
        rv.push(' ');
        rv.push_str(&self.expressions.join(", "));

        if let FromType::Specified(table) = self.from {
            rv.push(' ');
            rv.push_str("FROM");
            rv.push(' ');
            rv.push_str(table);
        }

        if !self.where_cl.is_empty() {
           rv.push(' ');
           rv.push_str("WHERE");
           rv.push(' ');
           rv.push_str(&self.where_cl.iter()
                       .map(|x| x.to_sql())
                       .collect::<Vec<_>>()
                       .join(" AND "));
        }

        match self.returning {
            Returning::Empty => {},
            Returning::All => {
                rv.push_str(RETURNING);
                rv.push('*');
            },
            Returning::Specified(ref values) => {
                rv.push_str(RETURNING);
                rv.push_str(&values.join(", "));
            }
        };

        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{FromType, Returning, Update};
    use common::ToSQL;
    use where_cl::{Where, IntoWhereType};
    use select::Select;

    #[test]
    fn smoke_test_builder() {
        let _upd = Update::new("test_table")
            .set("a = 2")
            .set(&["b = 3", "c = 5"])
            .filter("a == 10")
            .from("yo")
            .remove_from()
            .empty_returning()
            .returning_all()
            .returning("blah")
            .returning("ko");
    }

    #[test]
    fn test_simple() {
        let update = Update {
            table: "test_table",
            expressions: vec!["a = 2", "b = 3"],
            from: FromType::Empty,
            where_cl: vec![],
            returning: Returning::Empty
        };

        let built = Update::new("test_table").set("a = 2").set("b = 3");

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3".to_string()
        };

        assert!(update == built);
        assert_eq!(built.to_sql(), expected);
    }

    #[test]
    fn test_returning_all() {
        let update = Update {
            table: "test_table",
            expressions: vec!["a = 2", "b = 3"],
            from: FromType::Specified("other_test_table"),
            where_cl: vec!["d == 3".into_where_type()],
            returning: Returning::All
        };

        let built = Update::new("test_table")
            .set(&["a = 2", "b = 3"])
            .from("other_test_table")
            .filter("d == 3")
            .returning_all();

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3 \
            FROM other_test_table \
            WHERE d == 3 \
            RETURNING *".to_string()
        };

        assert!(update == built);
        assert_eq!(built.to_sql(), expected);
    }

    #[test]
    fn test_returning_some() {
        let foo = Where::with_and().expr("foo == bar").expr("fizz == bazz");
        let bar = Where::with_and().expr("a == b").expr("c == d");
        let where_cl = Where::with_or().expr(foo).expr(bar);

        let update = Update {
            table: "test_table",
            expressions: vec!["a = 2", "b = 3"],
            from: FromType::Empty,
            where_cl: vec![where_cl.clone().into_where_type()],
            returning: Returning::Specified(vec!["a", "b"])
        };

        let built = Update::new("test_table")
            .set(&["a = 2", "b = 3"])
            .filter(where_cl)
            .returning("a")
            .returning(&["b"]);

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3 \
            WHERE \
            ((foo == bar AND fizz == bazz) OR \
            (a == b AND c == d)) \
            RETURNING a, b".to_string()
        };

        assert!(update == built);
        assert_eq!(built.to_sql(), expected);
    }

    #[test]
    fn test_from_subquery() {
        let subquery = Select::from("blah_table").as_subquery().with_alias("alias");
        let update = Update::new("test_table").set("foo = bar").from(&subquery);
        let expected = {
            "UPDATE test_table SET foo = bar FROM (SELECT * FROM blah_table) AS alias".to_string()
        };
        assert_eq!(update.to_sql(), expected);
    }
}
