use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

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

    use super::ProfileState;

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
