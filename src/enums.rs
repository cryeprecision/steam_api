#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersonaState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    Invisible = 7,
}

impl PersonaState {
    pub const fn new(value: i32) -> Option<Self> {
        match value {
            0 => Some(PersonaState::Offline),
            1 => Some(PersonaState::Online),
            2 => Some(PersonaState::Busy),
            3 => Some(PersonaState::Away),
            4 => Some(PersonaState::Snooze),
            5 => Some(PersonaState::LookingToTrade),
            6 => Some(PersonaState::LookingToPlay),
            7 => Some(PersonaState::Invisible),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommunityVisibilityState {
    Private = 1,
    FriendsOnly = 2,
    Public = 3,
}

impl CommunityVisibilityState {
    pub const fn new(value: i32) -> Option<Self> {
        match value {
            1 => Some(CommunityVisibilityState::Private),
            2 => Some(CommunityVisibilityState::FriendsOnly),
            3 => Some(CommunityVisibilityState::Public),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EconomyBan {
    None,
    Probation,
    Banned,
    Unknown(String),
}

impl From<String> for EconomyBan {
    fn from(str: String) -> Self {
        match str.as_str() {
            "none" => EconomyBan::None,
            "probation" => EconomyBan::Probation,
            "banned" => EconomyBan::Banned,
            _ => EconomyBan::Unknown(str),
        }
    }
}
