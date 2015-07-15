pub enum Operator {
    And,
    Or
}

impl Operator {
    pub fn to_sql(&self) -> &str {
        match *self {
            Operator::And => "AND",
            Operator::Or => "OR"
        }
    }
}

trait ToSQL {
    fn to_sql(&self) -> String;
}

pub struct Where<'a, T: 'a> {
    pub operator: Operator,
    pub clause: &'a [T]
}

impl<'a> ToSQL for &'a str {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl<'a, T: ToSQL> ToSQL for Where<'a, T> {
    fn to_sql(&self) -> String {
        let operator = &format!(" {} ", self.operator.to_sql());
        let mut rv = String::new();
        rv.push('(');
        rv.push_str(&self.clause.into_iter()
                    .map(|x| x.to_sql())
                    .collect::<Vec<_>>()
                    .join(operator));
        rv.push(')');
        rv
    }
}

impl<'a, T: ToSQL> ToSQL for &'a Where<'a, T>{ 
    fn to_sql(&self) -> String {
        (**self).to_sql()
    }
}


#[cfg(test)]
mod tests {
    use super::{Operator, ToSQL, Where};

    #[test]
    fn test_operator() {
        let and = Operator::And;
        let or = Operator::Or;

        assert_eq!(and.to_sql(), "AND");
        assert_eq!(or.to_sql(), "OR");
    }

    #[test]
    fn test_alone_where() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo=bar", "fizz=bazz"]
        };
        assert_eq!(foo.to_sql(), "(foo=bar AND fizz=bazz)".to_string())
    }

    #[test]
    fn test_nested_where_clauses() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo == bar", "fizz == bazz"]
        };

        let bar = Where {
            operator: Operator::And,
            clause: &["a == b", "c == d"]
        };

        let bazz = Where {
            operator: Operator::Or,
            clause: &[&foo, &bar]
        };

        assert_eq!(bazz.to_sql(), "((foo == bar AND fizz == bazz) OR (a == b AND c == d))".to_string());
    }

    #[test]
    fn test_really_nested_where_clauses() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo == bar", "fizz == bazz"]
        };

        let bar = Where {
            operator: Operator::And,
            clause: &["a == b", "c == d"]
        };

        let bazz1 = Where {
            operator: Operator::Or,
            clause: &[&foo, &bar]
        };

        let bazz2 = Where {
            operator: Operator::Or,
            clause: &[&bar, &foo]
        };

        let fizz = Where {
            operator: Operator::And,
            clause: &[&bazz1, &bazz2]
        };

        assert_eq!(fizz.to_sql(), "(((foo == bar AND fizz == bazz) OR (a == b AND c == d)) AND ((a == b AND c == d) OR (foo == bar AND fizz == bazz)))".to_string());
    }
}
