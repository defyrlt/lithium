use common::Pusheable;

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
pub struct For<'a> {
    pub mode: ForMode,
    pub tables: Vec<&'a str>,
    pub nowait: bool,
}

impl<'a> For<'a> {
    pub fn new(mode: ForMode) -> Self {
        For {
            mode: mode,
            tables: vec![],
            nowait: false
        }
    }

    pub fn update() -> Self {
        Self::new(ForMode::Update)
    }

    pub fn share() -> Self {
        Self::new(ForMode::Share)
    }

    pub fn table<T: Pusheable<&'a str>>(mut self, tables: T) -> Self {
        tables.push_to(&mut self.tables);
        self
    }

    pub fn nowait(mut self) -> Self {
        self.nowait = true;
        self
    }
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

#[derive(Clone, PartialEq, Eq)]
pub enum ForType<'a> {
    Empty,
    Specified(For<'a>)
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
            tables: vec![],
            nowait: false
        };

        let built = For::update();

        assert!(for_cl == built);
        assert_eq!(for_cl.to_sql(), "FOR UPDATE")
    }

    #[test]
    fn test_for_with_clause() {
        let for_cl = For {
            mode: ForMode::Share,
            tables: vec!["foo", "bar"],
            nowait: false
        };

        let built = For::share().table(&["foo", "bar"]);

        assert!(for_cl == built);
        assert_eq!(for_cl.to_sql(), "FOR SHARE OF foo, bar")
    }

    #[test]
    fn test_for_with_clause_and_nowait() {
        let for_cl = For {
            mode: ForMode::Update,
            tables: vec!["foo", "bar"],
            nowait: true
        };

        let built = For::update().table("foo").table("bar").nowait();

        assert!(for_cl == built);
        assert_eq!(for_cl.to_sql(), "FOR UPDATE OF foo, bar NOWAIT")
    }
}
