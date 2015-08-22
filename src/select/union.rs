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
    fn new(mode: UnionMode, left: L, right: R) -> Self {
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
    use select::{Select, SelectType, DistinctType, LimitType, OffsetType, ForType};

    #[test]
    fn test_simple() {
        let query = Select {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

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
        let query = Select {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

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
        let query = Select {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

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
        let query = Select {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

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
