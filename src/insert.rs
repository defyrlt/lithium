use query::{ToSQL, Query};

#[allow(dead_code)]
pub enum Values<'a> {
    Default,
    Specified(&'a [&'a str]),
    Query(&'a Query<'a>)
}

#[allow(dead_code)]
impl<'a> Values<'a> {
    fn to_sql(&self) -> String {
        match *self {
            Values::Default => "DEFAULT VALUES".to_string(),
            Values::Specified(values) => {
                let mut rv = String::new();
                rv.push_str("VALUES");
                rv.push(' ');
                rv.push_str(&values.into_iter()
                            .map(|x| format!("({})", x))
                            .collect::<Vec<_>>()
                            .join(", "));
                rv
            },
            Values::Query(query) => query.to_sql()
        }
    }
}

#[allow(dead_code)]
pub enum Returning<'a> {
    All,
    Specified(&'a [&'a str])
}

#[allow(dead_code)]
impl<'a> Returning<'a> {
    pub fn to_sql(&self) -> String {
        match *self {
            Returning::All => "*".to_string(),
            Returning::Specified(values) => {
                format!("{}", values.join(", "))
            }
        }
    }
}

#[allow(dead_code)]
struct Insert<'a> {
    table: &'a str,
    columns: &'a [&'a str],
    values: Values<'a>,
    returning: Returning<'a>
}

#[allow(dead_code)]
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
        rv.push(' ');
        rv.push_str("RETURNING");
        rv.push(' ');
        rv.push_str(&self.returning.to_sql());
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{Values, Insert, Returning};

    use query::{ToSQL, Query};
    use select::SelectType;
    use where_cl::WhereType;
    use distinct::DistinctType;
    use limit::LimitType;
    use offset::OffsetType;
    use for_cl::ForType;

    #[test]
    fn test_simple() {
        let insert = Insert {
            table: "test_table",
            columns: &[],
            values: Values::Default,
            returning: Returning::All
        };

        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES \
            RETURNING *"
        };

        assert_eq!(insert.to_sql(), expected)
    }

    #[test]
    fn test_with_spec_return() {
        let returning_values = ["foo, bar"];
        let insert = Insert {
            table: "test_table",
            columns: &[],
            values: Values::Default,
            returning: Returning::Specified(&returning_values)
        };

        let expected = {
            "INSERT INTO test_table \
            DEFAULT VALUES \
            RETURNING foo, bar"
        };

        assert_eq!(insert.to_sql(), expected)
    }

    #[test]
    fn test_with_values() {
        let values = ["DEFAULT, fizz", "foo, bar"];
        let insert = Insert {
            table: "test_table",
            columns: &["foo", "bar"],
            values: Values::Specified(&values),
            returning: Returning::All
        };

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            VALUES (DEFAULT, fizz), (foo, bar) \
            RETURNING *"
        };

        assert_eq!(insert.to_sql(), expected)
    }

    #[test]
    fn test_with_query() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let insert = Insert {
            table: "test_table",
            columns: &["foo", "bar"],
            values: Values::Query(&query),
            returning: Returning::All
        };

        let expected = {
            "INSERT INTO test_table (foo, bar) \
            SELECT * FROM test_table \
            RETURNING *"
        };
        

        assert_eq!(insert.to_sql(), expected)
    }
}
