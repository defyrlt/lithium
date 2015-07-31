use query::Query;
use select::SelectType;
use join::{Join, JoinType};
use order_by::OrderBy;
use where_cl::WhereType;
use distinct::DistinctType;
use limit::LimitType;
use offset::OffsetType;
use for_cl::ForType;

#[allow(dead_code)]
pub struct Builder<'a> {
    pub select: SelectType<'a>,
    pub distinct: DistinctType<'a>,
    pub from: &'a str,
    pub joins: Vec<Join<'a>>,
    pub group_by: Vec<&'a str>,
    pub order_by: Vec<&'a OrderBy<'a>>,
    pub where_cl: WhereType<'a>,
    pub having: WhereType<'a>,
    pub limit: LimitType<'a>,
    pub offset: OffsetType<'a>,
    pub for_cl: ForType<'a>
}

#[allow(dead_code)]
impl<'a> Builder<'a> {
    fn from(table: &str) -> Builder {
        Builder {
            select: SelectType::All,
            distinct: DistinctType::Empty,
            from: table,
            joins: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            where_cl: WhereType::Empty,
            having: WhereType::Empty,
            limit: LimitType::Empty,
            offset: OffsetType::Empty,
            for_cl: ForType::Empty
        }
    }

    fn build(&'a self) -> Query<'a> {
        Query {
            select: &self.select,
            distinct: &self.distinct,
            from: &self.from,
            joins: &self.joins,
            group_by: &self.group_by,
            order_by: &self.order_by,
            where_cl: &self.where_cl,
            having: &self.having,
            limit: &self.limit,
            offset: &self.offset,
            for_cl: &self.for_cl
        }
    }

    fn select_all(&'a mut self) -> &'a mut Builder {
         self.select = SelectType::All;
         self
    }

    fn select(&'a mut self, fields: &'a [&'a str]) -> &'a mut Builder {
        self.select = SelectType::Specific(fields);
        self
    }

    fn clear_distinct(&'a mut self) -> &'a mut Builder {
        self.distinct = DistinctType::Empty;
        self
    }

    fn distinct(&'a mut self) -> &'a mut Builder {
        self.distinct = DistinctType::Simple;
        self
    }

    fn distinct_on(&'a mut self, fields: &'a [&'a str]) -> &'a mut Builder {
        self.distinct = DistinctType::Extended(fields);
        self
    }

    fn join(&'a mut self, target: &'a str, clause: &'a str) -> &'a mut Builder {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            target: target,
            clause: clause,
        });
        self
    }

    // fn left_join(&'a mut self, target: &'a str, clause: &'a str) -> &'a mut Builder {
    //     self.joins.push(&Join {
    //         join_type: &JoinType::Left,
    //         target: target,
    //         clause: clause,
    //     });
    //     self
    // }

    // fn right_join(&'a mut self, target: &'a str, clause: &'a str) -> &'a mut Builder {
    //     self.joins.push(Join {
    //         join_type: &JoinType::Right,
    //         target: target,
    //         clause: clause,
    //     });
    //     self
    // }

    // fn outer_join(&'a mut self, target: &'a str, clause: &'a str) -> &'a mut Builder {
    //     self.joins.push(Join {
    //         join_type: &JoinType::Outer,
    //         target: target,
    //         clause: clause,
    //     });
    //     self
    // }
}

#[cfg(test)]
mod tests {
    use super::Builder;
    use query::ToSQL;

    #[test]
    fn test_simple() {
        let builder = Builder::from("test_table");
        let query1 = builder.build();
        let query2 = builder.build();
        assert_eq!(query1.to_sql(), query2.to_sql());
    }
}
