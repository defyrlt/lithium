pub enum OffsetType<'a> {
    Empty,
    Specified(&'a str)
}