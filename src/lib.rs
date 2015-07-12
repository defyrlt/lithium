pub enum SelectType {
    All,
    Specific(Vec<String>),
}

impl SelectType {
    fn to_sql(&self) -> String {
        match *self {
            SelectType::All => "*".to_string(),
            SelectType::Specific(ref clauses) => clauses.connect(", ")
        }
    }
}

pub struct Query {
    select: SelectType,
    from: String
}

impl Query {
    fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("SELECT ");
        rv.push_str(&self.select.to_sql());
        rv.push(' ');
        rv.push_str("FROM ");
        rv.push_str(&self.from);
        rv.push(';');
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_all() {
        let query = Query {
            select: SelectType::All,
            from: "test_table".to_string()
        };
        assert_eq!(query.to_sql(), "SELECT * FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let query = Query {
            select: SelectType::Specific(vec!["foo".to_string(), "bar".to_string()]),
            from: "test_table".to_string()
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }
}
