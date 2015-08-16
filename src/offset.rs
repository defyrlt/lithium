#[derive(Clone, PartialEq, Eq)]
pub enum OffsetType<'a> {
    Empty,
    Specified(&'a str)
}
