#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocationTag {
    Pond,
    Ocean,
    River,
    Mines,
    Farm,
    Town,
    Other(String),
}

impl LocationTag {
    pub(crate) fn parse(value: &str) -> Self {
        match value {
            "pond" => Self::Pond,
            "ocean" => Self::Ocean,
            "river" => Self::River,
            "mines" => Self::Mines,
            "farm" => Self::Farm,
            "town" => Self::Town,
            _ => Self::Other(value.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Pond => "pond",
            Self::Ocean => "ocean",
            Self::River => "river",
            Self::Mines => "mines",
            Self::Farm => "farm",
            Self::Town => "town",
            Self::Other(value) => value,
        }
    }
}
