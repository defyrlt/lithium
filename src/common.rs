pub trait ToSQL {
    fn to_sql(&self) -> String;
}

pub trait AsStr<'a> {
    fn as_str(&self) -> &'a str;
}

impl<'a> AsStr<'a> for &'a str {
    fn as_str(&self) -> &'a str {
        *self
    }
}

impl<'a> AsStr<'a> for &'a Subquery<'a> {
    fn as_str(&self) -> &'a str {
        &self.query
    }
}

pub trait Pusheable<'a> {
    fn push_to(&self, destination: &mut Vec<&'a str>);
}

impl<'a> Pusheable<'a> for &'a str {
    fn push_to(&self, destination: &mut Vec<&'a str>) {
        destination.push(*self);
    }
}

impl<'a> Pusheable<'a> for &'a Subquery<'a> {
    fn push_to(&self, destination: &mut Vec<&'a str>) {
        destination.push(self.as_str())
    }
}

macro_rules! pusheable_impls {
    ($($N: expr)+) => {
        $(
            impl<'a, 'b> Pusheable<'a> for &'b [&'a str; $N] {
                fn push_to(&self, destination: &mut Vec<&'a str>) {
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


#[derive(Clone)]
pub struct Subquery<'a> {
    pub query: String,
    alias: Option<&'a str> // FIXME: do we need this?
}

impl<'a> Subquery<'a> {
    pub fn new(query: String) -> Self {
        Subquery {
            query: format!("({})", query),
            alias: None
        }
    }

    pub fn with_alias(mut self, alias: &'a str) -> Self {
        self.alias = Some(alias);
        self.query.push_str(&format!(" AS {}", alias));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Subquery;

    fn test_subquery() {
        let subquery = Subquery::new("blah".to_string());
        assert_eq!(subquery.query, "(blah)".to_string());
    }

    fn test_subquery_with_alias() {
        let subquery = Subquery::new("blah".to_string()).with_alias("foo");
        assert_eq!(subquery.query, "(blah) AS foo".to_string());
    }
}
