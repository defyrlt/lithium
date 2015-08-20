use query::{ToSQL, Query, Pusheable};

#[derive(Clone, PartialEq, Eq)]
pub enum Values<'a> {
    Default,
    Specified(Vec<&'a str>),
    Query(Query<'a>)
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
                            .map(|x| format!("({})", x))
                            .collect::<Vec<_>>()
                            .join(", "));
                rv
            },
            Values::Query(ref query) => query.to_sql()
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Returning<'a> {
    Empty,
    All,
    Specified(Vec<&'a str>)
}

impl<'a> Returning<'a> {
    pub fn to_sql(&self) -> String {
        match *self {
            Returning::All => "*".to_string(),
            Returning::Specified(ref values) => values.join(", "),
            Returning::Empty => unreachable!()
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Insert<'a> {
    table: &'a str,
    columns: Vec<&'a str>,
    values: Values<'a>,
    returning: Returning<'a>
}

impl<'a> Insert<'a> {
    pub fn new(table: &'a str) -> Self {
       Insert {
           table: table,
           columns: vec![],
           values: Values::Default,
           returning: Returning::Empty,
       }
    }

    pub fn columns<T: Pusheable<&'a str>>(mut self, columns: T) -> Self {
        columns.push_to(&mut self.columns);
        self
    }

    pub fn values<T: Pusheable<&'a str>>(mut self, input_values: T) -> Self {
        match self.values {
            Values::Default | Values::Query(_) => {
                let mut values = vec![];
                input_values.push_to(&mut values);
                self.values = Values::Specified(values);
            },
            Values::Specified(ref mut values) => input_values.push_to(values)
        }
        self
    }

    pub fn query(mut self, query: Query<'a>) -> Self {
        self.values = Values::Query(query);
        self
    }

    pub fn clear_returning(mut self) -> Self {
        self.returning = Returning::Empty;
        self
    }

    pub fn returning_all(mut self) -> Self {
        self.returning = Returning::All;
        self
    }

    pub fn returning<T: Pusheable<&'a str>>(mut self, input_fields: T) -> Self {
        match self.returning {
            Returning::Empty | Returning::All => {
                let mut fields = vec![];
                input_fields.push_to(&mut fields);
                self.returning = Returning::Specified(fields);
            },
            Returning::Specified(ref mut fields) => input_fields.push_to(fields)
        };
        self
    }
}

impl<'a> Insert<'a> {
    fn to_sql(&self) -> String {
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
            Returning::All | Returning::Specified(_) => {
                rv.push(' ');
                rv.push_str("RETURNING");
                rv.push(' ');
                rv.push_str(&self.returning.to_sql());
            },
            Returning::Empty => {}
        }

        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{Values, Insert, Returning};
    use query::Query;

    #[test]
    fn test_simple() {
        let insert = Insert {
            table: "test_table",
            columns: vec![],
            values: Values::Default,
            returning: Returning::Empty,
        };

        let built = Insert::new("test_table");

        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES".to_string()
        };

        assert!(insert == built);
        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_spec_return() {
        let insert = Insert {
            table: "test_table",
            columns: vec![],
            values: Values::Default,
            returning: Returning::Specified(vec!["foo", "bar"])
        };

        let built = Insert::new("test_table").returning("foo").returning("bar");

        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES \
            RETURNING foo, bar"
        };

        assert!(insert == built);
        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_values() {
        let insert = Insert {
            table: "test_table",
            columns: vec!["foo", "bar"],
            values: Values::Specified(vec!["DEFAULT, fizz", "foo, bar"]),
            returning: Returning::All
        };

        let built = Insert::new("test_table")
            .columns("foo")
            .columns(&["bar"])
            .values("DEFAULT, fizz")
            .values(&["foo, bar"])
            .returning_all();

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            VALUES (DEFAULT, fizz), (foo, bar) \
            RETURNING *"
        };

        assert!(insert == built);
        assert_eq!(insert.to_sql(), expected);
    }

    #[test]
    fn test_with_query() {
        let query = Query::new("test_table");
        let insert = Insert {
            table: "test_table",
            columns: vec!["foo", "bar"],
            values: Values::Query(query.clone()),
            returning: Returning::Specified(vec!["bar", "foo"])
        };

        let built = Insert::new("test_table")
            .columns(&["foo", "bar"])
            .query(query)
            .returning(&["bar", "foo"]);

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            SELECT * FROM test_table \
            RETURNING bar, foo"
        };
        
        assert!(insert == built);
        assert_eq!(insert.to_sql(), expected);
    }
}
