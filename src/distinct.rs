pub enum DistinctType<'a> {
    Empty,
    Simple,
    Extended(&'a [&'a str])
}
