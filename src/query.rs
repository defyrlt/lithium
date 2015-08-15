use select::SelectType;
use join::Join;
use order_by::OrderBy;
use where_cl::WhereType;
use distinct::DistinctType;
use limit::LimitType;
use offset::OffsetType;
use for_cl::ForType;

pub trait ToSQL {
    fn to_sql(&self) -> String;
}

#[derive(Clone)]
pub struct Query<'a> {
    pub select: SelectType<'a>,
    pub distinct: DistinctType<'a>,
    pub from: &'a str,
    pub joins: Vec<Join<'a>>,
    pub group_by: Vec<&'a str>,
    pub order_by: Vec<OrderBy<'a>>,
    pub where_cl: Vec<WhereType<'a>>,
    pub having: Vec<WhereType<'a>>,
    pub limit: LimitType<'a>,
    pub offset: OffsetType<'a>,
    pub for_cl: ForType<'a>
}

impl<'a> ToSQL for Query<'a> {
    fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("SELECT");

        match self.distinct {
            DistinctType::Empty => {},
            DistinctType::Simple => {
                rv.push(' ');
                rv.push_str("DISTINCT");
            },
            DistinctType::Extended(ref clauses) => {
                rv.push(' ');
                rv.push_str("DISTINCT ON");
                rv.push(' ');
                rv.push('(');
                rv.push_str(&clauses.join(", "));
                rv.push(')');
            }
        }

        rv.push(' ');
        rv.push_str(&self.select.to_sql());
        rv.push(' ');
        rv.push_str("FROM");
        rv.push(' ');
        rv.push_str(self.from);
        
        for join in &self.joins {
            rv.push(' ');
            rv.push_str(&join.to_sql());
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

        if !self.group_by.is_empty() {
            rv.push(' ');
            rv.push_str("GROUP BY");
            rv.push(' ');
            rv.push_str(&self.group_by.join(", "));
        }

        if !self.having.is_empty() {
           rv.push(' ');
           rv.push_str("HAVING");
           rv.push(' ');
           rv.push_str(&self.having.iter()
                       .map(|x| x.to_sql())
                       .collect::<Vec<_>>()
                       .join("AND"));
        }
        
        if !self.order_by.is_empty() {
            rv.push(' ');
            rv.push_str("ORDER BY");
            rv.push(' ');
            rv.push_str(&self.order_by
                        .iter()
                        .map(|x| x.to_sql())
                        .collect::<Vec<String>>()
                        .join(", "));
        }

        match self.limit {
            LimitType::Empty => {},
            LimitType::Specified(clause) => {
                rv.push(' ');
                rv.push_str("LIMIT");
                rv.push(' ');
                rv.push_str(clause);
            }
        }

        match self.offset {
            OffsetType::Empty => {},
            OffsetType::Specified(clause) => {
                rv.push(' ');
                rv.push_str("OFFSET");
                rv.push(' ');
                rv.push_str(clause);
            }
        }

        match self.for_cl {
            ForType::Empty => {},
            ForType::Specified(ref for_clause) => {
                rv.push(' ');
                rv.push_str(&for_clause.to_sql())
            }
        }

        rv
    }
}

impl<'a> ToSQL for &'a Query<'a> {
    fn to_sql(&self) -> String {
        (**self).to_sql()
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use self::test::Bencher;

    use super::{ToSQL, Query};
    use select::SelectType;
    use join::{JoinType, Join};
    use order_by::{Ordering, OrderBy};
    use where_cl::{Operator, Where, IntoWhereType};
    use distinct::DistinctType;
    use limit::LimitType;
    use offset::OffsetType;
    use for_cl::{ForMode, For, ForType};

    #[test]
    fn select_all() {
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
        assert_eq!(query.to_sql(), "SELECT * FROM test_table".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let query = Query {
            select: SelectType::Specific(vec!["foo", "bar"]),
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
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table".to_string());
    }

    #[test]
    fn select_foo_and_bar_with_vec_params() {
        let query = Query {
            select: SelectType::Specific(vec!["foo", "bar"]),
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
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table".to_string());
    }


    #[test]
    fn select_foo_and_bar_with_vec_params_and_strings() {
        let query = Query {
            select: SelectType::Specific(vec!["foo", "bar"]),
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
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table".to_string());
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
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![join],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN target_table ON 2 == 2".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
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
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![bar_join, bazz_join],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_group_by_foo() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec!["foo"],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_group_by_foo_and_bar() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec!["foo", "bar"],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo, bar".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_order_by() {
        let order_by_foo_asc = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "foo"
        };

        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![order_by_foo_asc],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_multi_order_by() {
        let order_by_foo_asc = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "foo"
        };

        let order_by_bar_desc = OrderBy {
            ordering: Ordering::Descending,
            order_by: "bar"
        };

        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![order_by_foo_asc, order_by_bar_desc],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC, bar DESC".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_where_simple() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec!["foo == bar".into_where_type()],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE foo == bar".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_where_extended() {
        let where_cl = Where::new(Operator::And).clause("foo == bar").clause("lala == blah");

        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![where_cl.into_where_type()],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE (foo == bar AND lala == blah)".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_with_having() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec!["foo == bar".into_where_type()],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING foo == bar".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_with_extended_having() {
        let where_cl = Where::new(Operator::And).clause("foo == bar").clause("lala == blah");

        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![where_cl.into_where_type()],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING (foo == bar AND lala == blah)".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_distinct_simple() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Simple,
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

        let test_sql_string = {
            "SELECT DISTINCT * \
            FROM test_table".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_distinct_extended() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Extended(vec!["foo", "bar"]),
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

        let test_sql_string = {
            "SELECT DISTINCT ON (foo, bar) * \
            FROM test_table".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_for_update() {
        let for_foo = For {
            mode: ForMode::Update,
            tables: vec![],
            nowait: false
        };

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
            for_cl: ForType::Specified(for_foo)
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            FOR UPDATE".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_for_update_clause() {
        let for_foo = For {
            mode: ForMode::Update,
            tables: vec!["foo", "bar"],
            nowait: false
        };

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
            for_cl: ForType::Specified(for_foo)
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            FOR UPDATE OF foo, bar".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn test_complex() {
        let for_bazz = For {
            mode: ForMode::Update,
            tables: vec![&"foo", &"bar"],
            nowait: true
        };

        let where_cl = Where::new(Operator::And).clause("foo == bar").clause("lala == blah");

        let order_by_bar_desc = OrderBy {
            ordering: Ordering::Descending,
            order_by: "bar"
        };

        let order_by_foo_asc = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "foo"
        };

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
            select: SelectType::Specific(vec!["foo", "bar"]),
            distinct: DistinctType::Extended(vec!["fizz", "bazz"]),
            from: "test_table",
            joins: vec![bar_join, bazz_join],
            group_by: vec!["foo", "bar"],
            order_by: vec![order_by_bar_desc, order_by_foo_asc],
            where_cl: vec![where_cl.clone().into_where_type()],
            having: vec![where_cl.clone().into_where_type()],
            limit: LimitType::Specified("10"),
            offset: OffsetType::Specified("5"),
            for_cl: ForType::Specified(for_bazz)
        };

        let test_sql_string = {
            "SELECT DISTINCT ON (fizz, bazz) foo, bar \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2 \
            WHERE (foo == bar AND lala == blah) \
            GROUP BY foo, bar \
            HAVING (foo == bar AND lala == blah) \
            ORDER BY bar DESC, foo ASC \
            LIMIT 10 \
            OFFSET 5 \
            FOR UPDATE OF foo, bar NOWAIT".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[bench]
    fn bench_query_with_extended_where(b: &mut Bencher) {
        let where_cl = Where::new(Operator::And).clause("foo == bar").clause("lala == blah");

        let order_by_bar_desc = OrderBy {
            ordering: Ordering::Descending,
            order_by: "bar"
        };

        let order_by_foo_asc = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "foo"
        };

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
            select: SelectType::Specific(vec!["foo", "bar"]),
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![bar_join, bazz_join],
            group_by: vec!["foo", "bar"],
            order_by: vec![order_by_bar_desc, order_by_foo_asc],
            where_cl: vec![where_cl.into_where_type()],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        b.iter(|| query.to_sql());
    }

    #[bench]
    fn bench_query_with_empty_where(b: &mut Bencher) {
        let order_by_bar_desc = OrderBy {
            ordering: Ordering::Descending,
            order_by: "bar"
        };

        let order_by_foo_asc = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "foo"
        };

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
            select: SelectType::Specific(vec!["foo", "bar"]),
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![bar_join, bazz_join],
            group_by: vec!["foo", "bar"],
            order_by: vec![order_by_bar_desc, order_by_foo_asc],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        b.iter(|| query.to_sql());
    }
}
