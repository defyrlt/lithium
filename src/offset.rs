#[allow(dead_code)]
pub enum OffsetType<'a> {
    Empty,
    Specified(&'a str)
}
