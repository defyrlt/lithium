pub enum Ordering {
    Ascending,
    Descending,
}

impl Ordering {
    pub fn to_sql(&self) -> &str {
        match *self {
            Ordering::Ascending => "ASC",
            Ordering::Descending => "DESC"
        }
    }
}

pub struct OrderBy<'a> {
    pub ordering: Ordering,
    pub order_by: &'a str
}

impl<'a> OrderBy<'a> {
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str(self.order_by);
        rv.push(' ');
        rv.push_str(self.ordering.to_sql());
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::{OrderBy, Ordering};

    #[test]
    fn test_ordering() {
        let ascending = Ordering::Ascending;
        let descending = Ordering::Descending;

        assert_eq!(ascending.to_sql(), "ASC");
        assert_eq!(descending.to_sql(), "DESC");
    }

    #[test]
    fn test_order_by() {
        let order_by = OrderBy {
            ordering: Ordering::Ascending,
            order_by: "fizz"
        };
        assert_eq!(order_by.to_sql(), "fizz ASC")
    }
}
