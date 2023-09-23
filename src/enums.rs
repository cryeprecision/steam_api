use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

pub enum EnumError<T> {
    Unknown(T),
}

/// <https://developer.valvesoftware.com/wiki/Steam_Web_API#Public_Data>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl TryFrom<i64> for PersonaState {
    type Error = EnumError<i64>;
    fn try_from(value: i64) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(PersonaState::Offline),
            1 => Ok(PersonaState::Online),
            2 => Ok(PersonaState::Busy),
            3 => Ok(PersonaState::Away),
            4 => Ok(PersonaState::Snooze),
            5 => Ok(PersonaState::LookingToTrade),
            6 => Ok(PersonaState::LookingToPlay),
            7 => Ok(PersonaState::Invisible),
            _ => Err(EnumError::Unknown(value)),
        }
    }
}

struct PersonaStateVisitor;

impl<'de> Visitor<'de> for PersonaStateVisitor {
    type Value = PersonaState;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("persona state enum variant as an integer")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        PersonaState::try_from(v)
            .map_err(|_| de::Error::invalid_value(Unexpected::Signed(v), &self))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let signed = i64::try_from(v)
            .map_err(|_| de::Error::invalid_value(Unexpected::Unsigned(v), &self))?;
        self.visit_i64(signed)
    }
}

impl<'de> Deserialize<'de> for PersonaState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i64(PersonaStateVisitor)
    }
}

/// <https://developer.valvesoftware.com/wiki/Steam_Web_API#Public_Data>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum CommunityVisibilityState {
    Private = 1,
    FriendsOnly = 2,
    Public = 3,
}

impl TryFrom<i64> for CommunityVisibilityState {
    type Error = EnumError<i64>;
    fn try_from(value: i64) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(CommunityVisibilityState::Private),
            2 => Ok(CommunityVisibilityState::FriendsOnly),
            3 => Ok(CommunityVisibilityState::Public),
            _ => Err(EnumError::Unknown(value)),
        }
    }
}

struct CommunityVisibilityStateVisitor;

impl<'de> Visitor<'de> for CommunityVisibilityStateVisitor {
    type Value = CommunityVisibilityState;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("persona state enum variant as an integer")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CommunityVisibilityState::try_from(v)
            .map_err(|_| de::Error::invalid_value(Unexpected::Signed(v), &self))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let signed = i64::try_from(v)
            .map_err(|_| de::Error::invalid_value(Unexpected::Unsigned(v), &self))?;
        self.visit_i64(signed)
    }
}

impl<'de> Deserialize<'de> for CommunityVisibilityState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i64(CommunityVisibilityStateVisitor)
    }
}

/// Undocumented 👻
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum EconomyBan {
    None,
    Probation,
    Banned,
}

impl<'a> TryFrom<&'a str> for EconomyBan {
    type Error = EnumError<&'a str>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(EconomyBan::None),
            "probation" => Ok(EconomyBan::Probation),
            "banned" => Ok(EconomyBan::Banned),
            _ => Err(EnumError::Unknown(value)),
        }
    }
}

struct EconomyBanVisitor;

impl<'de> Visitor<'de> for EconomyBanVisitor {
    type Value = EconomyBan;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("persona state enum variant as a string or byte sequence")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        EconomyBan::try_from(v).map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &self))
    }
}

impl<'de> Deserialize<'de> for EconomyBan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(EconomyBanVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ProfileState {
    Configured,
    NotConfigured,
}

struct ProfileStateVisitor;

impl<'de> Visitor<'de> for ProfileStateVisitor {
    type Value = ProfileState;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("profile state enum as either nothing or the number 1")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ProfileState::NotConfigured)
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i64(ProfileStateVisitor)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            1 => Ok(ProfileState::Configured),
            _ => Err(de::Error::invalid_value(Unexpected::Signed(v), &self)),
        }
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let signed = i64::try_from(v)
            .map_err(|_| de::Error::invalid_value(Unexpected::Unsigned(v), &self))?;
        self.visit_i64(signed)
    }
}

impl<'de> Deserialize<'de> for ProfileState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(ProfileStateVisitor)
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::{CommunityVisibilityState, EconomyBan, PersonaState, ProfileState};

    #[test]
    fn deserialize_economy_ban() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            economy_bans: Vec<EconomyBan>,
        }

        let json = serde_json::json!({
            "economy_bans": ["none", "probation", "banned"],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut states = parsed.economy_bans.into_iter();
        assert_eq!(states.next(), Some(EconomyBan::None));
        assert_eq!(states.next(), Some(EconomyBan::Probation));
        assert_eq!(states.next(), Some(EconomyBan::Banned));
    }

    #[test]
    fn deserialize_community_vis_state() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            vis_states: Vec<CommunityVisibilityState>,
        }

        let json = serde_json::json!({
            "vis_states": [1, 2, 3],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut states = parsed.vis_states.into_iter();
        assert_eq!(states.next(), Some(CommunityVisibilityState::Private));
        assert_eq!(states.next(), Some(CommunityVisibilityState::FriendsOnly));
        assert_eq!(states.next(), Some(CommunityVisibilityState::Public));
    }

    #[test]
    fn deserialize_persona_state() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            persona_states: Vec<PersonaState>,
        }

        let json = serde_json::json!({
            "persona_states": [0, 1, 2, 3, 4, 5, 6, 7],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut states = parsed.persona_states.into_iter();
        assert_eq!(states.next(), Some(PersonaState::Offline));
        assert_eq!(states.next(), Some(PersonaState::Online));
        assert_eq!(states.next(), Some(PersonaState::Busy));
        assert_eq!(states.next(), Some(PersonaState::Away));
        assert_eq!(states.next(), Some(PersonaState::Snooze));
        assert_eq!(states.next(), Some(PersonaState::LookingToTrade));
        assert_eq!(states.next(), Some(PersonaState::LookingToPlay));
        assert_eq!(states.next(), Some(PersonaState::Invisible));
    }

    #[test]
    fn deserialize_profile_state_some() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            profile_state: ProfileState,
        }

        let json = serde_json::json!({
            "profile_state": 1,
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let state = parsed.profile_state;
        assert_eq!(state, ProfileState::Configured);
    }

    #[test]
    fn deserialize_profile_state_none() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            profile_state: ProfileState,
        }

        let json = serde_json::json!({}).to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let state = parsed.profile_state;
        assert_eq!(state, ProfileState::NotConfigured);
    }
}
