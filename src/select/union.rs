use common::ToSQL;

pub enum UnionMode {
    Simple,
    All
}

pub struct Union<L: ToSQL, R: ToSQL> {
    left: L,
    right: R,
    mode: UnionMode
}

impl<L: ToSQL, R:ToSQL> Union<L, R> {
    /// Creates `Union` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithium::{ToSQL, Select};
    /// use lithium::select::{UnionMode, Union};
    ///
    /// let foo = Select::from("foo");
    /// let bar = Select::from("bar");
    /// // you can pass queries either by value or reference
    /// let union = Union::new(UnionMode::Simple, foo, &bar);
    /// let expected = {
    ///     "SELECT * FROM foo \
    ///     UNION \
    ///     SELECT * FROM bar".to_string()
    /// };
    /// assert_eq!(union.to_sql(), expected);
    /// ```
    ///
    /// ```
    /// use lithium::{ToSQL, Select};
    /// use lithium::select::{UnionMode, Union};
    ///
    /// let foo = Select::from("foo");
    /// let bar = Select::from("bar");
    /// let union = Union::new(UnionMode::Simple, &foo, &bar);
    /// let moar = Union::new(UnionMode::All, &union, &bar);
    /// let expected = {
    ///     "SELECT * FROM foo \
    ///     UNION \
    ///     SELECT * FROM bar \
    ///     UNION ALL \
    ///     SELECT * FROM bar".to_string()
    /// };
    /// assert_eq!(moar.to_sql(), expected);
    /// ```
    pub fn new(mode: UnionMode, left: L, right: R) -> Self {
        Union {
            mode: mode,
            left: left,
            right: right
        }
    }
}

impl<L: ToSQL, R: ToSQL> ToSQL for Union<L, R> {
    fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str(&self.left.to_sql());
        rv.push(' ');
        rv.push_str("UNION");
        rv.push(' ');

        if let UnionMode::All = self.mode {
            rv.push_str("ALL");
            rv.push(' ');
        }

        rv.push_str(&self.right.to_sql());
        rv
    }
}

impl<'a, L: ToSQL, R:ToSQL> ToSQL for &'a Union<L, R> {
    fn to_sql(&self) -> String {
        (**self).to_sql()
    }
}

#[cfg(test)]
mod tests {
    use super::{Union, UnionMode};
    use common::ToSQL;
    use select::Select;

    #[test]
    fn test_simple() {
        let query = Select::from("test_table");
        let union = Union::new(UnionMode::Simple, &query, &query);

        let expected = {
            "SELECT * FROM test_table \
            UNION \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_owned_queries() {
        let query = Select::from("test_table");
        let union = Union::new(UnionMode::Simple, query.clone(), query);

        let expected = {
            "SELECT * FROM test_table \
            UNION \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_union_all() {
        let query = Select::from("test_table");
        let union = Union::new(UnionMode::All, &query, &query);

        let expected = {
            "SELECT * FROM test_table \
            UNION ALL \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_nested() {
        let query = Select::from("test_table");
        let pre_union = Union::new(UnionMode::Simple, &query, &query);
        let union = Union::new(UnionMode::All, pre_union, &query);

        let expected = {
            "SELECT * FROM test_table \
            UNION \
            SELECT * FROM test_table \
            UNION ALL \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }
}
