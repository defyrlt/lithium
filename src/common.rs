//! Keeps stuff (mostly traits) that is used (or is going to be) across different queries.

pub trait ToSQL {
    fn to_sql(&self) -> String;
}

/// Is used to build up methods which can receive either `&str` or `&Subquery`
/// in a convenient way. You can find examples in some of `Select`'s methods.
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

/// Is used to build up methods which can receive either `&str` or `&[&str; N]`
/// in a convenient way. You can find examples in some of `Select`'s methods.
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


/// Struct that is used to keep result from `to_sql` of some query.
/// If you use `with_alias` - keep in mind that it's changing content of
/// `query` in **irreversible** way.  
/// We do this because we need `&str` to have a nice
/// way of using subqueries and avoid forcing users to use `String` when they don't
/// really need to.
#[derive(Clone)]
pub struct Subquery<'a> {
    /// Keeps generated SQL
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

    /// Appends `AS {alias}` to the `query` field
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
