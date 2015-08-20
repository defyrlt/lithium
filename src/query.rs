use select::SelectType;
use join::{Join, JoinType};
use order_by::{OrderBy, Ordering};
use where_cl::{WhereType, IntoWhereType};
use distinct::DistinctType;
use limit::LimitType;
use offset::OffsetType;
use for_cl::{For, ForType};

pub trait ToSQL {
    fn to_sql(&self) -> String;
}

pub trait Pusheable<T: Clone> {
    fn push_to(&self, destination: &mut Vec<T>);
}

impl<T: Clone> Pusheable<T> for T {
    fn push_to(&self, destination: &mut Vec<T>) {
        destination.push(self.clone());
    }
}

macro_rules! pusheable_impls {
    ($($N: expr)+) => {
        $(
            impl<'a, T: Clone> Pusheable<T> for &'a [T; $N] {
                fn push_to(&self, destination: &mut Vec<T>) {
                    destination.extend(self.iter().cloned());
                }
            }   
        )+
    }
}

pusheable_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

#[derive(Clone, PartialEq, Eq)]
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

impl<'a> Query<'a> {
    pub fn new(from_table: &'a str) -> Self {
        Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: from_table,
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        }
    }

    pub fn select_all(mut self) -> Self {
        self.select = SelectType::All;
        self
    }

    pub fn select<T: Pusheable<&'a str>>(mut self, input_fields: T) -> Self {
        match self.select {
            SelectType::All => {
                let mut fields = vec![];
                input_fields.push_to(&mut fields);
                self.select = SelectType::Specific(fields);
            },
            SelectType::Specific(ref mut fields) => input_fields.push_to(fields)
        }
        self
    }

    pub fn clear_distinct(mut self) -> Self {
        self.distinct = DistinctType::Empty;
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = DistinctType::Simple;
        self
    }

    pub fn distinct_on<T: Pusheable<&'a str>>(mut self, input_fields: T) -> Self {
        match self.distinct {
            DistinctType::Empty | DistinctType::Simple => {
                let mut fields = vec![];
                input_fields.push_to(&mut fields);
                self.distinct = DistinctType::Extended(fields);
            },
            DistinctType::Extended(ref mut fields) => input_fields.push_to(fields)
        }
        self
    }

    pub fn push_join(mut self, join_type: JoinType, target: &'a str, clause: &'a str) -> Self {
        self.joins.push(Join {
            join_type: join_type,
            target: target,
            clause: clause
        });
        self
    }

    pub fn join(self, target: &'a str, clause: &'a str) -> Self {
        self.push_join(JoinType::Inner, target, clause)
    }

    pub fn left_join(self, target: &'a str, clause: &'a str) -> Self {
        self.push_join(JoinType::Left, target, clause)
    }

    pub fn right_join(self, target: &'a str, clause: &'a str) -> Self {
        self.push_join(JoinType::Right, target, clause)
    }

    pub fn outer_join(self, target: &'a str, clause: &'a str) -> Self {
        self.push_join(JoinType::Outer, target, clause)
    }

    pub fn group_by<T: Pusheable<&'a str>>(mut self, fields: T) -> Self {
        fields.push_to(&mut self.group_by);
        self
    }

    pub fn order_by(mut self, field: &'a str, ordering: Ordering) -> Self {
        self.order_by.push(OrderBy {
            ordering: ordering,
            order_by: field
        });
        self
    }

    pub fn where_cl<T: IntoWhereType<'a>>(mut self, clause: T) -> Self {
        self.where_cl.push(clause.into_where_type());
        self
    }

    pub fn having<T: IntoWhereType<'a>>(mut self, clause: T) -> Self {
        self.having.push(clause.into_where_type());
        self
    }

    pub fn limit(mut self, value: &'a str) -> Self {
        self.limit = LimitType::Specified(value);
        self
    }

    pub fn clear_limit(mut self) -> Self {
        self.limit = LimitType::Empty;
        self
    }

    pub fn offset(mut self, value: &'a str) -> Self {
        self.offset = OffsetType::Specified(value);
        self
    }

    pub fn clear_offset(mut self) -> Self {
        self.offset = OffsetType::Empty;
        self
    }

    pub fn clear_for(mut self) -> Self {
        self.for_cl = ForType::Empty;
        self
    }

    pub fn for_cl(mut self, for_cl: For<'a>) -> Self {
        self.for_cl = ForType::Specified(for_cl);
        self
    }
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
                       .join(" AND "));
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
                       .join(" AND "));
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

        let built = Query::new("test_table");

        assert!(query == built);
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

        let built = Query::new("test_table").select("foo").select("bar");

        assert!(query == built);
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

        let built = Query::new("test_table").join("target_table", "2 == 2");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN target_table ON 2 == 2".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table")
            .join("bar_table", "1 == 1")
            .left_join("bazz_table", "2 == 2");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").group_by("foo");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").group_by(&["foo", "bar"]);

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            GROUP BY foo, bar".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").order_by("foo", Ordering::Ascending);

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table")
            .order_by("foo", Ordering::Ascending)
            .order_by("bar", Ordering::Descending);

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            ORDER BY foo ASC, bar DESC".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").where_cl("foo == bar");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE foo == bar".to_string()
        };

        assert!(query == built);
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_where_extended() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec!["foo == bar".into_where_type(), "lala == blah".into_where_type()],
            having: vec![],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let built = Query::new("test_table").where_cl("foo == bar").where_cl("lala == blah");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            WHERE foo == bar AND lala == blah".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").having("foo == bar");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING foo == bar".to_string()
        };

        assert!(query == built);
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_with_extended_having() {
        let query = Query {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: "test_table",
            joins: vec![],
            group_by: vec![],
            order_by: vec![],
            where_cl: vec![],
            having: vec!["foo == bar".into_where_type(), "lala == blah".into_where_type()],
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        };

        let built = Query::new("test_table").having("foo == bar").having("lala == blah");

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            HAVING foo == bar AND lala == blah".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").distinct();

        let test_sql_string = {
            "SELECT DISTINCT * \
            FROM test_table".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").distinct_on("foo").distinct_on("bar");

        let test_sql_string = {
            "SELECT DISTINCT ON (foo, bar) * \
            FROM test_table".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").for_cl(For::update());

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            FOR UPDATE".to_string()
        };

        assert!(query == built);
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

        let built = Query::new("test_table").for_cl(For::update().table("foo").table("bar"));

        let test_sql_string = {
            "SELECT * \
            FROM test_table \
            FOR UPDATE OF foo, bar".to_string()
        };

        assert!(query == built);
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn test_complex() {
        let for_bazz = For {
            mode: ForMode::Update,
            tables: vec!["foo", "bar"],
            nowait: true
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

        let query = Query {
            select: SelectType::Specific(vec!["foo", "bar"]),
            distinct: DistinctType::Extended(vec!["fizz", "bazz"]),
            from: "test_table",
            joins: vec![bar_join, bazz_join],
            group_by: vec!["foo", "bar"],
            order_by: vec![order_by_bar_desc, order_by_foo_asc],
            where_cl: vec!["foo == bar".into_where_type(), "lala == blah".into_where_type()],
            having: vec!["foo == bar".into_where_type(), "lala == blah".into_where_type()],
            limit: LimitType::Specified("10"),
            offset: OffsetType::Specified("5"),
            for_cl: ForType::Specified(for_bazz)
        };

        let built = Query::new("test_table")
            .select(&["foo", "bar"])
            .distinct_on(&["fizz", "bazz"])
            .join("bar_table", "1 == 1")
            .left_join("bazz_table", "2 == 2")
            .group_by(&["foo", "bar"])
            .where_cl("foo == bar").where_cl("lala == blah")
            .having("foo == bar").having("lala == blah")
            .order_by("bar", Ordering::Descending)
            .order_by("foo", Ordering::Ascending)
            .limit("10")
            .offset("5")
            .for_cl(For::update().table(&["foo", "bar"]).nowait());

        let test_sql_string = {
            "SELECT DISTINCT ON (fizz, bazz) foo, bar \
            FROM test_table \
            INNER JOIN bar_table ON 1 == 1 \
            LEFT JOIN bazz_table ON 2 == 2 \
            WHERE foo == bar AND lala == blah \
            GROUP BY foo, bar \
            HAVING foo == bar AND lala == blah \
            ORDER BY bar DESC, foo ASC \
            LIMIT 10 \
            OFFSET 5 \
            FOR UPDATE OF foo, bar NOWAIT".to_string()
        };

        assert!(query == built);
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

    #[bench]
    fn bench_builder(b: &mut Bencher) {
        b.iter(|| {
            let _ = Query::new("test_table")
                .select(&["foo", "bar"])
                .distinct_on(&["fizz", "bazz"])
                .join("bar_table", "1 == 1")
                .left_join("bazz_table", "2 == 2")
                .group_by(&["foo", "bar"])
                .where_cl("foo == bar").where_cl("lala == blah")
                .having("foo == bar").having("lala == blah")
                .order_by("bar", Ordering::Descending)
                .order_by("foo", Ordering::Ascending)
                .limit("10")
                .offset("5")
                .for_cl(For::update().table(&["foo", "bar"]).nowait());
        });
    }
}
