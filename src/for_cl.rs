pub enum ForMode {
    Update,
    Share
}

impl ForMode {
    fn to_sql(&self) -> &str {
        match *self {
            ForMode::Update => "UPDATE",
            ForMode::Share => "SHARE"
        }
    }
}

pub struct For<'a> {
    pub mode: ForMode,
    pub tables: &'a [&'a str],
    pub nowait: bool,
}

impl<'a> For<'a> {
    pub fn to_sql(&self) -> String {
        let mut rv = String::new();
        rv.push_str("FOR");
        rv.push(' ');
        rv.push_str(self.mode.to_sql());

        if !self.tables.is_empty() {
            rv.push(' ');
            rv.push_str("OF");
            rv.push(' ');
            rv.push_str(&self.tables.join(", "));
        }

        if self.nowait {
            rv.push(' ');
            rv.push_str("NOWAIT");
        }
        rv
    }
}

pub enum ForType<'a> {
    Empty,
    Specified(&'a For<'a>)
}

#[cfg(test)]
mod tests {
    use super::{ForMode, For};

    #[test]
    fn test_modes() {
        let update = ForMode::Update;
        let share = ForMode::Share;

        assert_eq!(update.to_sql(), "UPDATE");
        assert_eq!(share.to_sql(), "SHARE");
    }

    #[test]
    fn test_for() {
        let for_cl = For {
            mode: ForMode::Update,
            tables: &[],
            nowait: false
        };

        assert_eq!(for_cl.to_sql(), "FOR UPDATE")
    }

    #[test]
    fn test_for_with_clause() {
        let for_cl = For {
            mode: ForMode::Share,
            tables: &["foo", "bar"],
            nowait: false
        };

        assert_eq!(for_cl.to_sql(), "FOR SHARE OF foo, bar")
    }

    #[test]
    fn test_for_with_clause_and_nowait() {
        let for_cl = For {
            mode: ForMode::Update,
            tables: &["foo", "bar"],
            nowait: true
        };

        assert_eq!(for_cl.to_sql(), "FOR UPDATE OF foo, bar NOWAIT")
    }
}
