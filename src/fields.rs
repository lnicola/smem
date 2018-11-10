use std::str::FromStr;

// TODO The variants should map 1-to-1 to the ProcessStatistics fields.
//      Logic for user name vs user ID should be pushed out from the
//      various methods of ProcessStatistics.
#[derive(Clone, Copy)]
pub enum Field {
    Pid,
    User,
    Pss,
    Rss,
    Uss,
    Swap,
    Cmdline,
}

impl Field {
    pub fn name(&self) -> &str {
        match self {
            Field::Pid => "Pid",
            Field::User => "User",
            Field::Pss => "Pss",
            Field::Rss => "Rss",
            Field::Uss => "Uss",
            Field::Swap => "Swap",
            Field::Cmdline => "Cmdline",
        }
    }
}

impl FromStr for Field {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pid" => Ok(Field::Pid),
            "user" => Ok(Field::User),
            "pss" => Ok(Field::Pss),
            "rss" => Ok(Field::Rss),
            "uss" => Ok(Field::Uss),
            "swap" => Ok(Field::Swap),
            "cmdline" => Ok(Field::Cmdline),
            _ => Err(format!("Unknown field: {}", s)),
        }
    }
}
