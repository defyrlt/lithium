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

pub trait ToSQL {
    fn to_sql(&self) -> String;
}

pub struct Where<'a, T: 'a + ToSQL> {
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

pub enum WhereType<'a, T: 'a + ToSQL> {
    Simple(&'a str),
    Extended(&'a Where<'a, T>),
    Empty
}

impl<'a, T: 'a + ToSQL> WhereType<'a, T> {
    pub fn to_sql(&self) -> String {
        match *self {
            WhereType::Simple(string) => string.to_sql(),
            WhereType::Extended(clause) => clause.to_sql(),
            WhereType::Empty => String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Operator, ToSQL, WhereType, Where};

    #[test]
    fn test_where_types() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo=bar", "lala=blah"]
        };

        let simple = WhereType::Simple("fizz=bazz");
        let extended = WhereType::Extended(&foo);
        let empty = WhereType::Empty;

        assert_eq!(simple.to_sql(), "fizz=bazz".to_string());
        assert_eq!(extended.to_sql(), "(foo=bar AND lala=blah)".to_string());
        assert_eq!(empty.to_sql().is_empty(), true);
    }

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
