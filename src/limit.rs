#[allow(dead_code)]
pub enum LimitType<'a> {
    Empty,
    Specified(&'a str)
}
