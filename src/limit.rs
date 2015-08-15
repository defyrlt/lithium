#[derive(Clone)]
pub enum LimitType<'a> {
    Empty,
    Specified(&'a str)
}
