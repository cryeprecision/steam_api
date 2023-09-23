use std::ops::Deref;

use chrono::{DateTime, Local, TimeZone, Utc};
use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SteamTime {
    inner: DateTime<Local>,
}

impl SteamTime {
    pub const fn into_inner(self) -> DateTime<Local> {
        self.inner
    }
}

impl Deref for SteamTime {
    type Target = DateTime<Local>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct SteamTimeVisitor;

impl<'de> Visitor<'de> for SteamTimeVisitor {
    type Value = SteamTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("utc timestamp")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let utc = Utc
            .timestamp_opt(v, 0)
            .single()
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Signed(v), &self))?;

        Ok(SteamTime { inner: utc.into() })
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

impl<'de> Deserialize<'de> for SteamTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i64(SteamTimeVisitor)
    }
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};
    use serde::{Deserialize, Serialize};

    use super::SteamTime;

    #[test]
    fn deserialize() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            time: SteamTime,
        }

        let json = serde_json::json!({
            "time": 1681963569,
        })
        .to_string();

        let expected = Utc.with_ymd_and_hms(2023, 04, 20, 04, 06, 09).unwrap();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let time = parsed.time.into_inner();
        assert_eq!(time, expected);
    }
}
