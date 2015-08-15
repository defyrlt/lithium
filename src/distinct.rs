#[derive(Clone)]
pub enum DistinctType<'a> {
    Empty,
    Simple,
    Extended(Vec<&'a str>)
}

