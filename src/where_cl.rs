pub enum Conjunction {
    And,
    Or
}

impl Conjunction {
    pub fn to_sql(&self) -> &str {
        match *self {
            Conjunction::And => " AND ",
            Conjunction::Or => " OR "
        }
    }
}

struct Where<'a> {
    pub conjunction: Conjunction,
    pub where_cl: &'a [&'a str]
}

impl<'a> Where<'a> {
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str(&self.where_cl
                    .connect(self.conjunction.to_sql()));
        rv
    }

    pub fn get_nested(&self) -> String {
        format!("({})", self.to_sql())
    }
}




#[cfg(test)]
mod tests {
    use super::{Conjunction, Where};

    #[test]
    fn test_conjunction() {
        let and = Conjunction::And;
        let or = Conjunction::Or;

        assert_eq!(and.to_sql(), " AND ");
        assert_eq!(or.to_sql(), " OR ");
    }

    #[test]
    fn test_where_clause() {
        let where_clause = Where {
            conjunction: Conjunction::And,
            where_cl: &["fizz = bazz", "lala = blah"]
        };
        assert_eq!(where_clause.to_sql(), "fizz = bazz AND lala = blah");
    }

    #[test]
    fn test_where_nested_clause() {
        let where_clause_foo = Where {
            conjunction: Conjunction::And,
            where_cl: &["foo = bar", "fizz = bazz"]
        };

        let where_clause_bar = Where {
            conjunction: Conjunction::Or,
            where_cl: &["None = Null"]
        };

        let where_nested = Where {
            conjunction: Conjunction::And,
            where_cl: &[&where_clause_foo.get_nested(),
                        &where_clause_bar.get_nested()]
        };
        assert_eq!(where_nested.to_sql(), "(foo = bar AND fizz = bazz) AND (None = Null)");
    }
}
