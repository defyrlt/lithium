#[derive(Clone, PartialEq, Eq)]
pub enum DistinctType<'a> {
    Empty,
    Simple,
    Extended(Vec<&'a str>)
}

