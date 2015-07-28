use where_cl::WhereType;

// TODO: make it pretty
const WHERE: &'static str = " WHERE ";
const RETURNING: &'static str = " RETURNING ";

#[allow(dead_code)]
enum FromType<'a> {
    Empty,
    Specified(&'a str)
}

#[allow(dead_code)]
enum ReturningType<'a> {
    Empty,
    All,
    Specified(&'a [&'a str])
}

#[allow(dead_code)]
struct Update<'a> {
    table: &'a str,
    expressions: &'a [&'a str],
    from: FromType<'a>,
    where_cl: WhereType<'a>,
    returning: ReturningType<'a>
}

impl<'a> Update<'a> {
    #[allow(dead_code)]
    fn to_sql(&self) -> String {
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

        match self.where_cl {
            WhereType::Empty => {},
            WhereType::Simple(value) => {
                rv.push_str(WHERE);
                rv.push_str(value);
            },
            WhereType::Extended(value) => {
                rv.push_str(WHERE);
                rv.push_str(&value.to_sql());
            }
        }

        match self.returning {
            ReturningType::Empty => {},
            ReturningType::All => {
                rv.push_str(RETURNING);
                rv.push('*');
            },
            ReturningType::Specified(values) => {
                rv.push_str(RETURNING);
                rv.push_str(&values.join(", "));
            }
        };

        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{FromType, ReturningType, Update};
    use where_cl::{WhereType, Operator, Where};

    #[test]
    fn test_simple() {
        let update = Update {
            table: "test_table",
            expressions: &["a = 2", "b = 3"],
            from: FromType::Empty,
            where_cl: WhereType::Empty,
            returning: ReturningType::Empty
        };

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3".to_string()
        };

        assert_eq!(update.to_sql(), expected);
    }

    #[test]
    fn test_returning_all() {
        let update = Update {
            table: "test_table",
            expressions: &["a = 2", "b = 3"],
            from: FromType::Specified("other_test_table"),
            where_cl: WhereType::Simple("d == 3"),
            returning: ReturningType::All
        };

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3 \
            FROM other_test_table \
            WHERE d == 3 \
            RETURNING *".to_string()
        };

        assert_eq!(update.to_sql(), expected);
    }

    #[test]
    fn test_returning_some() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo == bar", "fizz == bazz"]
        };

        let bar = Where {
            operator: Operator::And,
            clause: &["a == b", "c == d"]
        };

        let where_cl = Where {
            operator: Operator::Or,
            clause: &[&foo, &bar]
        };

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3 \
            FROM other_test_table \
            WHERE \
            ((foo == bar AND fizz == bazz) OR \
            (a == b AND c == d)) \
            RETURNING a, b".to_string()
        };

        let returning = ["a", "b"];
        let update = Update {
            table: "test_table",
            expressions: &["a = 2", "b = 3"],
            from: FromType::Specified("other_test_table"),
            where_cl: WhereType::Extended(&where_cl),
            returning: ReturningType::Specified(&returning)
        };

        assert_eq!(update.to_sql(), expected);
    }
}
