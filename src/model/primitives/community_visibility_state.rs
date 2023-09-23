use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

use super::EnumError;

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

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::CommunityVisibilityState;

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
        assert_eq!(states.next(), None);
    }
}
