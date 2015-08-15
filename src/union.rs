use query::ToSQL;

enum UnionType {
    Simple,
    All
}

struct Union<L: ToSQL, R: ToSQL> {
    left: L,
    right: R,
    mode: UnionType
}

impl<L: ToSQL, R: ToSQL> ToSQL for Union<L, R> {
    fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str(&self.left.to_sql());
        rv.push(' ');
        rv.push_str("UNION");
        rv.push(' ');

        if let UnionType::All = self.mode {
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
    use super::{Union, UnionType};
    use query::{ToSQL, Query};
    use select::SelectType;
    use distinct::DistinctType;
    use limit::LimitType;
    use offset::OffsetType;
    use for_cl::ForType;

    #[test]
    fn test_simple() {
        let query = Query {
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

        let union = Union {
            left: &query,
            right: &query,
            mode: UnionType::Simple
        };

        let expected = {
            "SELECT * FROM test_table \
            UNION \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_owned_queries() {
        let query = Query {
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

        let union = Union {
            left: query.clone(),
            right: query,
            mode: UnionType::Simple
        };

        let expected = {
            "SELECT * FROM test_table \
            UNION \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_union_all() {
        let query = Query {
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

        let union = Union {
            left: &query,
            right: &query,
            mode: UnionType::All
        };

        let expected = {
            "SELECT * FROM test_table \
            UNION ALL \
            SELECT * FROM test_table".to_string()
        };
        assert_eq!(union.to_sql(), expected);
    }

    #[test]
    fn test_nested() {
        let query = Query {
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

        let pre_union = Union {
            left: &query,
            right: &query,
            mode: UnionType::Simple
        };

        let union = Union {
            left: &pre_union,
            right: &query,
            mode: UnionType::All
        };

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
