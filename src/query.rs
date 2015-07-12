use select::SelectType;

pub struct Query<'a> {
    select: SelectType<'a>,
    from: &'a str
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
        rv.push(';');
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::Query;
    use select::SelectType;

    #[test]
    fn select_all() {
        let query = Query {
            select: SelectType::All,
            from: "test_table"
        };
        assert_eq!(query.to_sql(), "SELECT * FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let clauses = &["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(clauses),
            from: "test_table"
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar_with_vec_params() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table"
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }


    #[test]
    fn select_foo_and_bar_with_vec_params_and_strings() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table"
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }
}

