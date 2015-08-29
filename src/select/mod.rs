pub mod select_type;
pub mod distinct;
pub mod join;
pub mod order_by;
pub mod limit;
pub mod offset;
pub mod for_cl;
pub mod union;

use common::{ToSQL, AsStr, Pusheable, Subquery};
use where_cl::{WhereType, IntoWhereType};

pub use self::select_type::SelectType;
pub use self::join::{Join, JoinType};
pub use self::order_by::{OrderBy, Ordering};
pub use self::distinct::DistinctType;
pub use self::limit::LimitType;
pub use self::offset::OffsetType;
pub use self::for_cl::{For, ForType};

/// Represents `SELECT` query.
///
/// # Examples
/// Simple "select all":
///
/// ```
/// use lithium::{ToSQL, Select};
/// let query = Select::from("test_table");
/// assert_eq!(query.to_sql(), "SELECT * FROM test_table");
/// ```
///
/// Selecting some fields:
///
/// ```
/// use lithium::{ToSQL, Select};
/// // you can pass &str or &[&str] into `fields` method
/// let query = Select::from("test_table").fields("foo").fields(&["bar", "bazz"]);
/// assert_eq!(query.to_sql(), "SELECT foo, bar, bazz FROM test_table");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Select<'a> {
    pub select_type: SelectType<'a>,
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

impl<'a> Select<'a> {
    /// Method to start with.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithium::Select;
    /// let query = Select::from("test_table");
    /// ```
    ///
    /// You could pass a subquery into `from`
    ///
    /// ```
    /// use lithium::Select;
    /// let subquery = Select::from("foo_table").as_subquery().with_alias("foo");
    /// let query = Select::from(&subquery);
    /// ```
    pub fn from<T: AsStr<'a>>(from_table: T) -> Self {
        Select {
            select_type: SelectType::All,
            distinct: DistinctType::Empty,
            from: from_table.as_str(),
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

    /// Will result in `SELECT * ...` (which is a default behaviour).
    pub fn select_all(mut self) -> Self {
        self.select_type = SelectType::All;
        self
    }

    /// This method is used to specify desired `SELECT` fields.
    /// It can receive either `&str` or `&[&str]`
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Select;
    /// let query = Select::from("test_table").fields("blah").fields(&["foo", "bar"]);
    /// ```
    pub fn fields<T: Pusheable<'a>>(mut self, input_fields: T) -> Self {
        match self.select_type {
            SelectType::All => {
                let mut fields = vec![];
                input_fields.push_to(&mut fields);
                self.select_type = SelectType::Specific(fields);
            },
            SelectType::Specific(ref mut fields) => input_fields.push_to(fields)
        }
        self
    }

    /// Removes `DISTINCT` clause from query.
    pub fn remove_distinct(mut self) -> Self {
        self.distinct = DistinctType::Empty;
        self
    }

    /// Will result in `SELECT DISTINCT ...`
    pub fn distinct(mut self) -> Self {
        self.distinct = DistinctType::Simple;
        self
    }

    /// Will result in `SELECT DISTINCT ON (...) ...`
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Select;
    /// let query = Select::from("test_table").distinct_on("blah").distinct_on(&["foo", "bar"]);
    /// ```
    pub fn distinct_on<T: Pusheable<'a>>(mut self, input_fields: T) -> Self {
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

    fn push_join<T: AsStr<'a>>(mut self, join_type: JoinType, target: T, clause: &'a str) -> Self {
        self.joins.push(Join {
            join_type: join_type,
            target: target.as_str(),
            clause: clause.as_str(),
        });
        self
    }

    /// Appends `INNER JOIN` into query. Could receive a subquery as `target`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithium::Select;
    /// let query = Select::from("test_table")
    ///     .join("another_table", "another_table.a == test_table.a");
    /// ```
    ///
    /// ```
    /// use lithium::Select;
    /// let subquery = Select::from("test_table").as_subquery().with_alias("test");
    /// let query = Select::from("foo_table").join(&subquery, "test.a == foo_table.a");
    /// ```
    pub fn join<T: AsStr<'a>>(self, target: T, clause: &'a str) -> Self {
        self.push_join(JoinType::Inner, target, clause)
    }

    pub fn left_join<T: AsStr<'a>>(self, target: T, clause: &'a str) -> Self {
        self.push_join(JoinType::Left, target, clause)
    }

    pub fn right_join<T: AsStr<'a>>(self, target: T, clause: &'a str) -> Self {
        self.push_join(JoinType::Right, target, clause)
    }

    pub fn outer_join<T: AsStr<'a>>(self, target: T, clause: &'a str) -> Self {
        self.push_join(JoinType::Outer, target, clause)
    }

    /// Appends `GROUP BY` clause.
    /// This method can receive either `&str` or `&[&str]`
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Select;
    /// let query = Select::from("test_table").group_by("blah").group_by(&["foo", "bar"]);
    /// ```
    pub fn group_by<T: Pusheable<'a>>(mut self, fields: T) -> Self {
        fields.push_to(&mut self.group_by);
        self
    }

    /// Appends `ORDER BY` clause.
    ///
    /// # Example
    ///
    /// ```
    /// use lithium::Select;
    /// use lithium::select::Ordering;
    /// let query = Select::from("test_table").order_by("foo", Ordering::Ascending);
    /// ```
    pub fn order_by(mut self, field: &'a str, ordering: Ordering) -> Self {
        self.order_by.push(OrderBy {
            ordering: ordering,
            order_by: field
        });
        self
    }

    /// Appends `WHERE` clause.
    ///
    /// # Examples
    ///
    /// Clauses are connected with `AND` by default.
    ///
    /// ```
    /// use lithium::{Select, ToSQL};
    /// let query = Select::from("test_table").where_cl("foo == bar").where_cl("bar == bazz");
    /// assert_eq!(query.to_sql(), "SELECT * FROM test_table WHERE foo == bar AND bar == bazz".to_owned());
    /// ```
    ///
    /// That's how you can use `OR`:
    ///
    /// ```
    /// use lithium::{Select, Where};
    /// let query = Select::from("test_table")
    ///     .where_cl(Where::with_or().clause("foo == bar").clause("bar == bazz"));
    /// ```
    pub fn where_cl<T: IntoWhereType<'a>>(mut self, clause: T) -> Self {
        self.where_cl.push(clause.into_where_type());
        self
    }

    /// Appends `HAVING` clause. Has the same API and usage as `where_cl`.
    pub fn having<T: IntoWhereType<'a>>(mut self, clause: T) -> Self {
        self.having.push(clause.into_where_type());
        self
    }

    /// Appends `LIMIT` clause.
    pub fn limit(mut self, value: &'a str) -> Self {
        self.limit = LimitType::Specified(value);
        self
    }

    /// Removes `LIMIT` clause.
    pub fn remove_limit(mut self) -> Self {
        self.limit = LimitType::Empty;
        self
    }

    /// Appends `OFFSET` clause.
    pub fn offset(mut self, value: &'a str) -> Self {
        self.offset = OffsetType::Specified(value);
        self
    }

    /// Removes `OFFSET` clause.
    pub fn remove_offset(mut self) -> Self {
        self.offset = OffsetType::Empty;
        self
    }

    /// Appends `FOR` clause.
    pub fn for_cl(mut self, for_cl: For<'a>) -> Self {
        self.for_cl = ForType::Specified(for_cl);
        self
    }

    /// Removes `FOR` clause.
    pub fn remove_for(mut self) -> Self {
        self.for_cl = ForType::Empty;
        self
    }

    /// Returns an instance of `Subquery` with generated SQL inside.
    pub fn as_subquery(self) -> Subquery<'a> {
        Subquery::new(self.to_sql())
    }
}

impl<'a> ToSQL for Select<'a> {
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
        rv.push_str(&self.select_type.to_sql());
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

impl<'a> ToSQL for &'a Select<'a> {
    fn to_sql(&self) -> String {
        (**self).to_sql()
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use self::test::Bencher;

    use common::{ToSQL};
    use where_cl::{Where, IntoWhereType};

    use super::Select;
    use super::select_type::SelectType;
    use super::join::{JoinType, Join};
    use super::order_by::{Ordering, OrderBy};
    use super::distinct::DistinctType;
    use super::limit::LimitType;
    use super::offset::OffsetType;
    use super::for_cl::{ForMode, For, ForType};

    #[test]
    fn select_all() {
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table");

        assert!(query == built);
        assert_eq!(query.to_sql(), "SELECT * FROM test_table".to_string());
    }

    #[test]
    fn select_foo_and_bar() {
        let query = Select {
            select_type: SelectType::Specific(vec!["foo", "bar"]),
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

        let built = Select::from("test_table").fields("foo").fields("bar");

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").join("target_table", "2 == 2");

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table")
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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").group_by("foo");

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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").group_by(&["foo", "bar"]);

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").order_by("foo", Ordering::Ascending);

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table")
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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").where_cl("foo == bar");

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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").where_cl("foo == bar").where_cl("lala == blah");

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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").having("foo == bar");

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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").having("foo == bar").having("lala == blah");

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
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").distinct();

        let test_sql_string = {
            "SELECT DISTINCT * \
            FROM test_table".to_string()
        };

        assert!(query == built);
        assert_eq!(query.to_sql(), test_sql_string);
    }

    #[test]
    fn select_all_distinct_extended() {
        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").distinct_on("foo").distinct_on("bar");

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").for_cl(For::update());

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

        let query = Select {
            select_type: SelectType::All,
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

        let built = Select::from("test_table").for_cl(For::update().table("foo").table("bar"));

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

        let query = Select {
            select_type: SelectType::Specific(vec!["foo", "bar"]),
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

        let built = Select::from("test_table")
            .fields(&["foo", "bar"])
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

    #[test]
    fn test_subquery() {
        let subquery = Select::from("test_table").as_subquery().with_alias("blah");
        let another = Select::from("test_table").fields(&subquery);
        let test_sql_string = {
            "SELECT (SELECT * FROM test_table) AS blah FROM test_table".to_string()
        };
        assert_eq!(another.to_sql(), test_sql_string);
    }

    #[test]
    fn test_join_on_subquery() {
        let subquery = Select::from("foo_table").as_subquery().with_alias("bar");
        let another = Select::from("bazz_table").join(&subquery, "bar.a == bazz_table.a");
        let test_sql_string = {
            "SELECT * FROM bazz_table INNER JOIN \
            (SELECT * FROM foo_table) AS bar ON bar.a == bazz_table.a".to_string()
        };
        assert_eq!(another.to_sql(), test_sql_string);
    }

    #[test]
    fn test_select_from_subquery() {
        let subquery = Select::from("foo_table").as_subquery().with_alias("bar");
        let another = Select::from(&subquery);
        let test_sql_string = {
            "SELECT * FROM (SELECT * FROM foo_table) AS bar".to_string()
        };
        assert_eq!(another.to_sql(), test_sql_string);
    }

    #[bench]
    fn bench_query_with_extended_where(b: &mut Bencher) {
        let where_cl = Where::with_and().clause("foo == bar").clause("lala == blah");

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

        let query = Select {
            select_type: SelectType::Specific(vec!["foo", "bar"]),
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

        let query = Select {
            select_type: SelectType::Specific(vec!["foo", "bar"]),
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
            let _ = Select::from("test_table")
                .fields(&["foo", "bar"])
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
