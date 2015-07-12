use select::SelectType;
use join::Join;

pub struct Query<'a> {
    pub select: SelectType<'a>,
    pub from: &'a str,
    pub joins: &'a [Join<'a>]
}

impl<'a> Query<'a> {
    fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("SELECT");
        rv.push(' ');
        rv.push_str(&self.select.to_sql());
        rv.push(' ');
        rv.push_str("FROM");
        rv.push(' ');
        rv.push_str(self.from);
        
        for join in self.joins {
            rv.push(' ');
            rv.push_str(&join.to_sql());
        }

        rv.push(';');
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::Query;
    use select::SelectType;
    use join::{JoinType, Join};

    #[test]
    fn select_all() {
        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[]
        };
        assert_eq!(query.to_sql(), "SELECT * FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let clauses = &["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(clauses),
            from: "test_table",
            joins: &[]
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar_with_vec_params() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[]
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }


    #[test]
    fn select_foo_and_bar_with_vec_params_and_strings() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[]
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_join_bar() {
        let join = Join {
            join_type: JoinType::Inner,
            target: "target_table",
            clause: "2 == 2"
        };

        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[join],
        };

        assert_eq!(query.to_sql(), "SELECT * FROM test_table INNER JOIN target_table ON 2 == 2;");
    }

    #[test]
    fn select_foo_and_join_bar_and_bazz() {
        let bar_join = Join {
            join_type: JoinType::Inner,
            target: "bar_table",
            clause: "1 == 1"
        };

        let bazz_join = Join {
            join_type: JoinType::Left,
            target: "bazz_table",
            clause: "2 == 2"
        };

        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[bar_join, bazz_join],
        };

        assert_eq!(query.to_sql(), "SELECT * FROM test_table INNER JOIN bar_table ON 1 == 1 LEFT JOIN bazz_table ON 2 == 2;");
    }
}

