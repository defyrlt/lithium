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

struct Where<'a, T: 'a> {
    pub operator: Operator,
    pub clause: &'a [&'a T]
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
        rv.push_str(&self.clause.into_iter()
                    .map(|x| x.to_sql())
                    .collect::<Vec<_>>()
                    .connect(operator));
        rv
    }
}




#[cfg(test)]
mod tests {
    use super::{Operator, Where};

    #[test]
    fn test_operator() {
        let and = Operator::And;
        let or = Operator::Or;

        assert_eq!(and.to_sql(), "AND");
        assert_eq!(or.to_sql(), "OR");
    }

    #[test]
    fn test_generic() {
        let foo = Where {
            operator: Operator::And,
            clause: &["foo", "bar"]
        };
    }
}
