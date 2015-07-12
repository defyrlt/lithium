// TODO: add cross join
pub enum JoinType {
    Inner,
    Left,
    Right,
    Outer
}

impl JoinType {
    pub fn to_sql(&self) -> &str {
        match *self {
            JoinType::Inner => "INNER",
            JoinType::Left => "LEFT",
            JoinType::Right => "RIGHT",
            JoinType::Outer => "OUTER"
        }
    }
}

pub struct Join<'a> {
    pub join_type: JoinType,
    pub target: &'a str,
    pub clause: &'a str
}

impl<'a> Join<'a> {
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str(self.join_type.to_sql());
        rv.push(' ');
        rv.push_str("JOIN");
        rv.push(' ');
        rv.push_str(self.target);
        rv.push(' ');
        rv.push_str("ON");
        rv.push(' ');
        rv.push_str(self.clause);
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{JoinType, Join};

    #[test]
    fn test_join_types() {
        let inner = JoinType::Inner;
        let left = JoinType::Left;
        let right = JoinType::Right;
        let outer = JoinType::Outer;

        assert_eq!(inner.to_sql(), "INNER");
        assert_eq!(left.to_sql(), "LEFT");
        assert_eq!(right.to_sql(), "RIGHT");
        assert_eq!(outer.to_sql(), "OUTER");
    }

    #[test]
    fn test_join() {
        let join = Join {
            join_type: JoinType::Inner,
            target: "target_table",
            clause: "2 == 2"
        };
        assert_eq!(join.to_sql(), "INNER JOIN target_table ON 2 == 2");
    }
}
