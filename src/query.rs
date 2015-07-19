use select::SelectType;
use join::Join;
use order_by::OrderBy;
use where_cl::{ToSQL, WhereType};
use limit::LimitType;

pub struct Query<'a> {
    pub select: SelectType<'a>,
    pub from: &'a str,
    pub joins: &'a [&'a Join<'a>],
    pub group_by: &'a [&'a str],
    pub order_by: &'a [&'a OrderBy<'a>],
    pub where_cl: WhereType<'a>,
    pub having: WhereType<'a>,
    pub limit: LimitType<'a>
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

        let where_string = " WHERE ";
        match self.where_cl {
            WhereType::Empty => {},
            WhereType::Simple(clause) => {
                rv.push_str(where_string);
                rv.push_str(&clause.to_sql());
            },
            WhereType::Extended(clause) => {
                rv.push_str(where_string);
                rv.push_str(&clause.to_sql());
            }
        }

        if !self.group_by.is_empty() {
            rv.push(' ');
            rv.push_str("GROUP BY");
            rv.push(' ');
            rv.push_str(&self.group_by.join(", "));
        }

        let having_string = " HAVING ";
        match self.having {
            WhereType::Empty => {},
            WhereType::Simple(clause) => {
                rv.push_str(having_string);
                rv.push_str(&clause.to_sql());
            },
            WhereType::Extended(clause) => {
                rv.push_str(having_string);
                rv.push_str(&clause.to_sql());
            }
        }
        
        if !self.order_by.is_empty() {
            rv.push(' ');
            rv.push_str("ORDER BY");
            rv.push(' ');
            rv.push_str(&self.order_by
                        .into_iter()
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

        rv.push(';');
        rv
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;

    use super::Query;
    use select::SelectType;
    use join::{JoinType, Join};
    use order_by::{Ordering, OrderBy};
    use where_cl::{Operator, ToSQL, WhereType, Where};
    use limit::LimitType;

    #[test]
    fn select_all() {
        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };
        assert_eq!(query.to_sql(), "SELECT * FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let clauses = &["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(clauses),
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }

    #[test]
    fn select_foo_and_bar_with_vec_params() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };
        assert_eq!(query.to_sql(), "SELECT foo, bar FROM test_table;".to_string());
    }


    #[test]
    fn select_foo_and_bar_with_vec_params_and_strings() {
        let clauses = vec!["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
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
            joins: &[&join],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN target_table ON 2 == 2;".to_string()
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
            from: "test_table",
            joins: &[&bar_join, &bazz_join],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_group_by_foo() {
        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &["foo"],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_and_group_by_foo_and_bar() {
        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &["foo", "bar"],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo, bar;".to_string()
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
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[&order_by_foo_asc],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC;".to_string()
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
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[&order_by_foo_asc, &order_by_bar_desc],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC, bar DESC;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_where_simple() {
        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Simple("foo == bar"),
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE foo == bar;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_where_extended() {
        let where_cl = Where {
            operator: Operator::And,
            clause: &["foo == bar", "lala == blah"]
        };

        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Extended(&where_cl),
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE (foo == bar AND lala == blah);".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_with_having() {
        let where_cl = Where {
            operator: Operator::And,
            clause: &["foo == bar", "lala == blah"]
        };

        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Simple("foo == bar"),
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING foo == bar;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_with_extended_having() {
        let where_cl = Where {
            operator: Operator::And,
            clause: &["foo == bar", "lala == blah"]
        };

        let query = Query {
            select: SelectType::All,
            from: "test_table",
            joins: &[],
            group_by: &[],
            order_by: &[],
            where_cl: WhereType::Empty,
            having: WhereType::Extended(&where_cl),
            limit: LimitType::Empty,
        };

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING (foo == bar AND lala == blah);".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn test_complex() {
        let where_cl = Where {
            operator: Operator::And,
            clause: &["foo == bar", "lala == blah"]
        };

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

        let clauses = ["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[&bar_join, &bazz_join],
            group_by: &["foo", "bar"],
            order_by: &[&order_by_bar_desc, &order_by_foo_asc],
            where_cl: WhereType::Extended(&where_cl),
            having: WhereType::Extended(&where_cl),
            limit: LimitType::Specified("10")
        };

        let test_sql_string = {
            "SELECT foo, bar \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2 \
            WHERE (foo == bar AND lala == blah) \
            GROUP BY foo, bar \
            HAVING (foo == bar AND lala == blah) \
            ORDER BY bar DESC, foo ASC \
            LIMIT 10;".to_string()
        };
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[bench]
    fn bench_query_with_extended_where(b: &mut Bencher) {
        let where_cl = Where {
            operator: Operator::And,
            clause: &["foo == bar", "lala == blah"]
        };

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

        let clauses = ["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[&bar_join, &bazz_join],
            group_by: &["foo", "bar"],
            order_by: &[&order_by_bar_desc, &order_by_foo_asc],
            where_cl: WhereType::Extended(&where_cl),
            having: WhereType::Empty,
            limit: LimitType::Empty,
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

        let clauses = ["foo", "bar"];
        let query = Query {
            select: SelectType::Specific(&clauses),
            from: "test_table",
            joins: &[&bar_join, &bazz_join],
            group_by: &["foo", "bar"],
            order_by: &[&order_by_bar_desc, &order_by_foo_asc],
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
        };

        b.iter(|| query.to_sql());
    }
}
