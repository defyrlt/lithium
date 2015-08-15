#[derive(Clone)]
pub enum SelectType<'a> {
    All,
    Specific(Vec<&'a str>)
}

impl<'a> SelectType<'a> {
    // TODO: use `Cow` instead?
    pub fn to_sql(&self) -> String {
        match *self {
            SelectType::All => "*".to_string(),
            SelectType::Specific(ref clauses) => clauses.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectType;

    #[test]
    fn select_all() {
        let select = SelectType::All;
        assert_eq!(select.to_sql(), "*");
    }

    #[test]
    fn select_foo_and_bar() {
        let select = SelectType::Specific(vec!["foo", "bar"]);
        assert_eq!(select.to_sql(), "foo, bar".to_string());
    }

    #[test]
    fn select_foo_and_bar_with_vec_params() {
        let select = SelectType::Specific(vec!["foo", "bar"]);
        assert_eq!(select.to_sql(), "foo, bar".to_string());
    }
}
