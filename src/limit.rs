pub enum LimitType<'a> {
    Empty,
    Specified(&'a str)
}
