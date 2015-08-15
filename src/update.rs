use query::ToSQL;
use where_cl::{WhereType, IntoWhereType};

// TODO: make it pretty
const RETURNING: &'static str = " RETURNING ";

#[derive(Clone, PartialEq, Eq)]
enum FromType<'a> {
    Empty,
    Specified(&'a str)
}

#[derive(Clone, PartialEq, Eq)]
enum ReturningType<'a> {
    Empty,
    All,
    Specified(Vec<&'a str>)
}

#[derive(Clone, PartialEq, Eq)]
struct Update<'a> {
    table: &'a str,
    expressions: Vec<&'a str>,
    from: FromType<'a>,
    where_cl: Vec<WhereType<'a>>,
    returning: ReturningType<'a>
}

impl<'a> Update<'a> {
    fn new(table: &'a str) -> Self {
        Update {
            table: table,
            expressions: vec![],
            from: FromType::Empty,
            where_cl: vec![],
            returning: ReturningType::Empty
        }
    }

    fn expression(mut self, expression: &'a str) -> Self {
        self.expressions.push(expression);
        self
    }

    fn push_expressions(mut self, expressions: &'a [&'a str]) -> Self {
        self.expressions.extend(expressions.iter().cloned());
        self
    }

    fn where_cl<T: IntoWhereType<'a>>(mut self, clause: T) -> Self {
        self.where_cl.push(clause.into_where_type());
        self
    }

    fn from(mut self, table: &'a str) -> Self {
        self.from = FromType::Specified(table);
        self
    }

    fn clear_from(mut self) -> Self {
        self.from = FromType::Empty;
        self
    }

    fn empty_returning(mut self) -> Self {
        self.returning = ReturningType::Empty;
        self
    }

    fn returning_all(mut self) -> Self {
        self.returning = ReturningType::All;
        self
    }

    fn returning(mut self, expression: &'a str) -> Self {
        match self.returning {
            ReturningType::Empty | ReturningType::All => {
                self.returning = ReturningType::Specified(vec![expression]);
            },
            ReturningType::Specified(ref mut expressions) => expressions.push(expression)
        }
        self
    }

    fn push_returning(mut self, expressions: &'a [&'a str]) -> Self {
        let new = expressions.iter().cloned();
        match self.returning {
            ReturningType::Empty | ReturningType::All => {
                let mut returning = Vec::new();
                returning.extend(new);
                self.returning = ReturningType::Specified(returning);
            },
            ReturningType::Specified(ref mut returning) => returning.extend(new)
        }
        self
        
    }
}

impl<'a> ToSQL for Update<'a> {
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

        if !self.where_cl.is_empty() {
           rv.push(' ');
           rv.push_str("WHERE");
           rv.push(' ');
           rv.push_str(&self.where_cl.iter()
                       .map(|x| x.to_sql())
                       .collect::<Vec<_>>()
                       .join("AND"));
        }

        match self.returning {
            ReturningType::Empty => {},
            ReturningType::All => {
                rv.push_str(RETURNING);
                rv.push('*');
            },
            ReturningType::Specified(ref values) => {
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
    use query::ToSQL;
    use where_cl::{Operator, Where, IntoWhereType};

    #[test]
    fn smoke_test_builder() {
        let exprs = ["b = 3", "c = 5"];
        let _upd = Update::new("test_table")
            .expression("a = 2")
            .push_expressions(&exprs)
            .where_cl("a == 10")
            .from("yo")
            .clear_from()
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
            returning: ReturningType::Empty
        };

        let built = Update::new("test_table").expression("a = 2").expression("b = 3");

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
            returning: ReturningType::All
        };

        let built = Update::new("test_table")
            .expression("a = 2")
            .expression("b = 3")
            .from("other_test_table")
            .where_cl("d == 3")
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
        let foo = Where::new(Operator::And).clause("foo == bar").clause("fizz == bazz");
        let bar = Where::new(Operator::And).clause("a == b").clause("c == d");
        let where_cl = Where::new(Operator::Or).clause(foo).clause(bar);

        let update = Update {
            table: "test_table",
            expressions: vec!["a = 2", "b = 3"],
            from: FromType::Specified("other_test_table"),
            where_cl: vec![where_cl.clone().into_where_type()],
            returning: ReturningType::Specified(vec!["a", "b"])
        };

        let built = Update::new("test_table")
            .expression("a = 2")
            .expression("b = 3")
            .from("other_test_table")
            .where_cl(where_cl)
            .returning("a")
            .returning("b");

        let expected = {
            "UPDATE test_table \
            SET a = 2, b = 3 \
            FROM other_test_table \
            WHERE \
            ((foo == bar AND fizz == bazz) OR \
            (a == b AND c == d)) \
            RETURNING a, b".to_string()
        };

        assert!(update == built);
        assert_eq!(built.to_sql(), expected);
    }
}
