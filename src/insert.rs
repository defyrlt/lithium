//! Keeps `INSERT` related stuff.

use select::Select;
use common::{ToSQL, Pusheable};

// TODO: make it pretty
const RETURNING: &'static str = " RETURNING ";

#[derive(Clone)]
enum Values<'a> {
    Default,
    Specified(Vec<Vec<&'a str>>),
    Select(Select<'a>)
}

impl<'a> Values<'a> {
    fn to_sql(&self) -> String {
        match *self {
            Values::Default => "DEFAULT VALUES".to_string(),
            Values::Specified(ref values) => {
                let mut rv = String::new();
                rv.push_str("VALUES");
                rv.push(' ');
                rv.push_str(&values.iter()
                            .map(|x| format!("({})", x.join(", ")))
                            .collect::<Vec<_>>()
                            .join(", "));
                rv
            },
            Values::Select(ref query) => query.to_sql()
        }
    }
}

#[derive(Clone)]
enum Returning<'a> {
    Empty,
    All,
    Specified(Vec<&'a str>)
}

/// Represents `INSERT` query.
#[derive(Clone)]
pub struct Insert<'a> {
    table: &'a str,
    columns: Vec<&'a str>,
    values: Values<'a>,
    returning: Returning<'a>
}

impl<'a> Insert<'a> {
    /// Method to start with. 
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("test_table");
    /// let expected = "INSERT INTO test_table DEFAULT VALUES".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    pub fn into(table: &'a str) -> Self {
       Insert {
           table: table,
           columns: vec![],
           values: Values::Default,
           returning: Returning::Empty,
       }
    }

    /// Specifies columns for `INSERT`.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("test_table").columns(&["foo", "bar"]).columns("bazz");
    /// ```
    pub fn columns<T: Pusheable<'a>>(mut self, columns: T) -> Self {
        columns.push_to(&mut self.columns);
        self
    }

    /// Specifies `INSERT` values. Sorry for receiving `Vec` here - we're going to find a better way
    /// for this.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("foo").columns("bar").values(vec!["bazz"]);
    /// ```
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("foo").columns(&["bar", "bazz"])
    ///     .values(vec!["123", "123"]).values(vec!["345", "678"]); 
    /// let expected = "INSERT INTO foo (bar, bazz) VALUES (123, 123), (345, 678)".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    // pub fn values<T: Pusheable<'a>>(mut self, input_values: T) -> Self {
    //     match self.values {
    //         Values::Default | Values::Select(_) => {
    //             let mut values = vec![];
    //             input_values.push_to(&mut values);
    //             self.values = Values::Specified(values);
    //         },
    //         Values::Specified(ref mut values) => input_values.push_to(values)
    //     }
    //     self
    // }
    pub fn values(mut self, input_values: Vec<&'a str>) -> Self {
        match self.values {
            Values::Default | Values::Select(_) => {
                self.values = Values::Specified(vec![input_values]);
            },
            Values::Specified(ref mut values) => values.push(input_values)
        }
        self
    }

    /// Specifies `SELECT` as `INSERT` value. Results in `INSERT INTO ... SELECT`
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::{Select, Insert};
    /// let select = Select::from("foo").columns(&["a", "b"]);
    /// let insert = Insert::into("bar").columns(&["a", "b"]).query(select);
    /// let expected = "INSERT INTO bar (a, b) SELECT a, b FROM foo".to_string();
    /// assert_eq!(insert.to_sql(), expected);
    /// ```
    pub fn query(mut self, query: Select<'a>) -> Self {
        self.values = Values::Select(query);
        self
    }

    /// Specifies `RETURNING` clause. WIll result in `RETURNING *`
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("foo").values(vec!["bar"]).returning_all();
    /// let expected = "INSERT INTO foo VALUES (bar) RETURNING *".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    pub fn returning_all(mut self) -> Self {
        self.returning = Returning::All;
        self
    }

    /// Specifies columns for `RETURNING` clause.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Insert;
    /// let query = Insert::into("foo").values(vec!["bar", "bazz"]).returning(&["a", "b"]);
    /// let expected = "INSERT INTO foo VALUES (bar, bazz) RETURNING a, b".to_string();
    /// assert_eq!(query.to_sql(), expected);
    /// ```
    pub fn returning<T: Pusheable<'a>>(mut self, input_columns: T) -> Self {
        match self.returning {
            Returning::Empty | Returning::All => {
                let mut columns = vec![];
                input_columns.push_to(&mut columns);
                self.returning = Returning::Specified(columns);
            },
            Returning::Specified(ref mut columns) => input_columns.push_to(columns)
        };
        self
    }

    /// Removes `RETURNING` clause.
    pub fn remove_returning(mut self) -> Self {
        self.returning = Returning::Empty;
        self
    }

    /// Generates SQL.
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("INSERT INTO");
        rv.push(' ');
        rv.push_str(&self.table);

        if !self.columns.is_empty() {
            rv.push(' ');
            rv.push('(');
            rv.push_str(&self.columns.join(", "));
            rv.push(')');
        }

        rv.push(' ');
        rv.push_str(&self.values.to_sql());

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
    use super::{Values, Insert, Returning};
    use select::Select;

    #[test]
    fn test_simple() {
        let insert = Insert::into("test_table");
        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES".to_string()
        };
        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_spec_return() {
        let insert = Insert::into("test_table").returning("foo").returning("bar");
        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES \
            RETURNING foo, bar"
        };
        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_values() {
        let insert = Insert::into("test_table")
            .columns("foo")
            .columns(&["bar"])
            .values(vec!["DEFAULT, fizz"])
            .values(vec!["foo", "bar"])
            .returning_all();

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            VALUES (DEFAULT, fizz), (foo, bar) \
            RETURNING *"
        };

        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_query() {
        let query = Select::from("test_table");
        let insert = Insert::into("test_table")
            .columns(&["foo", "bar"])
            .query(query)
            .returning(&["bar", "foo"]);

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            SELECT * FROM test_table \
            RETURNING bar, foo"
        };
        
        assert_eq!(insert.to_sql(), expected);
    }
}
