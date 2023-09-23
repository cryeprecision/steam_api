use serde::de::Visitor;
use serde::{Deserialize, Serialize};

pub enum EnumError<T> {
    Unknown(T),
}

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
        PersonaState::try_from(v).map_err(|_| {
            E::custom(format!(
                "the number {} does not correspond to an enum variant",
                v
            ))
        })
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let signed = i64::try_from(v).map_err(|_| {
            E::custom(format!(
                "the number {} does not correspond to an enum variant",
                v,
            ))
        })?;
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
        CommunityVisibilityState::try_from(v).map_err(|_| {
            E::custom(format!(
                "{} does not correspond to a community visibility state",
                v
            ))
        })
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let signed = i64::try_from(v).map_err(|_| {
            E::custom(format!(
                "{} does not correspond to a community visibility state",
                v,
            ))
        })?;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum EconomyBan {
    None,
    Probation,
    Banned,
}

impl<'a> TryFrom<&'a [u8]> for EconomyBan {
    type Error = EnumError<&'a [u8]>;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"none" => Ok(EconomyBan::None),
            b"probation" => Ok(EconomyBan::Probation),
            b"banned" => Ok(EconomyBan::Banned),
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

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        EconomyBan::try_from(v)
            .map_err(|_| E::custom(format!("{:?} does not correspond to an economy ban", v)))
    }
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v.as_slice())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_borrowed_bytes(v.as_bytes())
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_borrowed_str(v.as_str())
    }
}

impl<'de> Deserialize<'de> for EconomyBan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(EconomyBanVisitor)
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::{CommunityVisibilityState, EconomyBan, PersonaState};

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
}
