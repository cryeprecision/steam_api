use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

use super::EnumError;

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

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::EconomyBan;

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
        assert_eq!(states.next(), None);
    }
}
