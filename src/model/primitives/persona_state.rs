use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

use super::EnumError;

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

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::PersonaState;

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
        assert_eq!(states.next(), None);
    }
}
