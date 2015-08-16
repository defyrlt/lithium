#[derive(Clone, PartialEq, Eq)]
pub enum LimitType<'a> {
    Empty,
    Specified(&'a str)
}
