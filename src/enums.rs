#[derive(Debug)]
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

impl TryFrom<i32> for PersonaState {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PersonaState::Offline),
            1 => Ok(PersonaState::Online),
            2 => Ok(PersonaState::Busy),
            3 => Ok(PersonaState::Away),
            4 => Ok(PersonaState::Snooze),
            5 => Ok(PersonaState::LookingToTrade),
            6 => Ok(PersonaState::LookingToPlay),
            7 => Ok(PersonaState::Invisible),
            _ => Err("invalid persona state"),
        }
    }
}

#[derive(Debug)]
pub enum CommunityVisibilityState {
    Private = 1,
    FriendsOnly = 2,
    Public = 3,
}

impl TryFrom<i32> for CommunityVisibilityState {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(CommunityVisibilityState::Private),
            2 => Ok(CommunityVisibilityState::FriendsOnly),
            3 => Ok(CommunityVisibilityState::Public),
            _ => Err("invalid community visibility state"),
        }
    }
}
