mod query_ext;
pub use query_ext::SteamIdQueryExt;
use serde::{Deserialize, Serialize};

#[cfg(feature = "friend_code")]
mod friend_code;

use std::fmt;
use std::fmt::Write;
use std::str::FromStr;

use crate::model::{AccountType, Universe};

/// Wrapper for [`SteamId`]s that is implemented according to [`Valve`](https://developer.valvesoftware.com/wiki/SteamID)
///
/// The bit shifting is explained here:
/// - [`As Represented in Computer Programs`](https://developer.valvesoftware.com/wiki/SteamID#As_Represented_in_Computer_Programs)
/// - [`Steam ID as a Steam Community ID`](https://developer.valvesoftware.com/wiki/SteamID#Steam_ID_as_a_Steam_Community_ID)
///
/// # From the Valve documentation
///
/// - The lowest bit represents `Y`.
/// - The next `31` bits represent the account number.
/// - The next `20` bits represent the instance of the account. It is usually set to `1` for user accounts.
/// - The next `4` bits represent the type of the account.
/// - The next `8` bits represent the universe the steam account belongs to.
/// - `X` represents the universe the steam account belongs to.
/// - `Y` is part of the ID number for the account, it is either `0` or `1`.
/// - `Z` is the account number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SteamId(pub u64);

/// Essentially the same as [`SteamId`] but serializes to a string and deserializes from a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SteamIdStr(pub u64);

impl From<SteamIdStr> for SteamId {
    fn from(value: SteamIdStr) -> Self {
        SteamId(value.0)
    }
}
impl From<SteamId> for SteamIdStr {
    fn from(value: SteamId) -> Self {
        SteamIdStr(value.0)
    }
}

impl SteamIdStr {
    pub fn steam_id(self) -> SteamId {
        self.into()
    }
}

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for SteamId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}
impl From<i64> for SteamId {
    fn from(id: i64) -> Self {
        Self(id.max(0) as u64)
    }
}

impl From<SteamId> for String {
    fn from(value: SteamId) -> Self {
        value.to_string()
    }
}

impl FromStr for SteamId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SteamId(s.parse()?))
    }
}

pub mod ser {

    use serde::{Serialize, Serializer};

    use super::SteamIdStr;

    impl Serialize for SteamIdStr {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.steam_id().to_string())
        }
    }
}

pub mod de {
    use std::borrow::Cow;
    use std::str::FromStr;

    use serde::{Deserialize, Deserializer};

    use super::SteamIdStr;
    use crate::SteamId;

    impl<'de> Deserialize<'de> for SteamIdStr {
        fn deserialize<D>(deserializer: D) -> Result<SteamIdStr, D::Error>
        where
            D: Deserializer<'de>,
        {
            let str = <Cow<'de, str>>::deserialize(deserializer)?;
            SteamId::from_str(&str)
                .map_err(serde::de::Error::custom)
                .map(SteamIdStr::from)
        }
    }
}

impl SteamId {
    const Y_SHIFT: u64 = 0;
    const Y_MASK: u64 = (1 << 1) - 1;

    const ACC_NR_SHIFT: u64 = 1;
    const ACC_NR_MASK: u64 = (1 << 31) - 1;

    const INSTANCE_SHIFT: u64 = 32;
    const INSTANCE_MASK: u64 = (1 << 20) - 1;

    const TYPE_SHIFT: u64 = 52;
    const TYPE_MASK: u64 = (1 << 4) - 1;

    const UNIVERSE_SHIFT: u64 = 56;
    const UNIVERSE_MASK: u64 = (1 << 8) - 1;

    /// Maximum number of decimal digits needed to represent a [`u64`].
    ///
    /// ```
    /// assert_eq!(((u64::MAX as f64).log10().floor() as usize) + 1, 20);
    /// ```
    ///
    /// <https://www.exploringbinary.com/number-of-decimal-digits-in-a-binary-integer/>
    pub const MAX_DIGITS_FOR_U64: usize = 20;

    /// <https://developer.valvesoftware.com/wiki/SteamID#Steam_ID_as_a_Steam_Community_ID#:~:text=W%3DZ*2%2BY>
    pub const fn w(&self) -> u64 {
        2 * self.acc_nr() + self.y()
    }

    pub const fn y(&self) -> u64 {
        (self.0 >> Self::Y_SHIFT) & Self::Y_MASK
    }

    pub const fn acc_nr(&self) -> u64 {
        (self.0 >> Self::ACC_NR_SHIFT) & Self::ACC_NR_MASK
    }

    pub const fn instance(&self) -> u64 {
        (self.0 >> Self::INSTANCE_SHIFT) & Self::INSTANCE_MASK
    }

    pub fn acc_type(&self) -> Option<AccountType> {
        let acc_type = (self.0 >> Self::TYPE_SHIFT) & Self::TYPE_MASK;
        AccountType::try_from(acc_type).ok()
    }

    pub fn universe(&self) -> Option<Universe> {
        let universe = (self.0 >> Self::UNIVERSE_SHIFT) & Self::UNIVERSE_MASK;
        Universe::try_from(universe).ok()
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// <https://developer.valvesoftware.com/wiki/SteamID#As_Represented_Textually>
    pub fn to_steam_id(&self) -> Option<String> {
        let x = self.universe()?.as_u64();
        let z = self.acc_nr();
        let mut buf = String::with_capacity("STEAM_X:X:XXXXXXXXXX".len());
        write!(buf, "STEAM_{}:{}:{}", x, self.y(), z).unwrap();
        Some(buf)
    }

    /// <https://developer.valvesoftware.com/wiki/SteamID#Steam_ID_as_a_Steam_Community_ID>
    pub fn to_steam_id_3(&self) -> Option<String> {
        let letter = self.acc_type()?.to_letter()?;
        let mut buf = String::with_capacity("[X:1:XXXXXXXXXX]".len());
        write!(buf, "[{}:1:{}]", letter, self.w()).unwrap();
        Some(buf)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::SteamId;
    use crate::steam_id::SteamIdStr;

    #[test]
    fn deserialize_steam_ids_str() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_ids: Vec<SteamIdStr>,
        }

        let json = serde_json::json!({
            "steam_ids": ["76561198805665689", "76561197992321696"],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut steam_ids = parsed.steam_ids.into_iter();
        assert_eq!(steam_ids.next(), Some(SteamIdStr(76561198805665689)));
        assert_eq!(steam_ids.next(), Some(SteamIdStr(76561197992321696)));
        assert_eq!(steam_ids.next(), None);
    }

    #[test]
    fn deserialize_steam_ids_int() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_ids: Vec<SteamId>,
        }

        let json = serde_json::json!({
            "steam_ids": [76561198805665689u64, 76561197992321696u64],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut steam_ids = parsed.steam_ids.into_iter();
        assert_eq!(steam_ids.next(), Some(SteamId(76561198805665689)));
        assert_eq!(steam_ids.next(), Some(SteamId(76561197992321696)));
        assert_eq!(steam_ids.next(), None);
    }

    #[test]
    fn deserialize_steam_id_int() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_id: SteamId,
        }

        let json = serde_json::json!({
            "steam_id": 76561198805665689u64,
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.steam_id, SteamId(76561198805665689));
    }

    #[test]
    fn serialize_to_string() {
        let serialized: String =
            serde_json::to_string(&SteamId(76561198805665689).to_string()).unwrap();
        assert_eq!(serialized, r#""76561198805665689""#);
    }

    #[test]
    fn deserialize_steam_id_str() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_id: SteamIdStr,
        }

        let json = serde_json::json!({
            "steam_id": "76561198805665689",
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.steam_id, SteamIdStr(76561198805665689));
    }

    #[test]
    fn to_steam_id() {
        let id = SteamId(76561198805665689);
        assert_eq!(id.to_steam_id().unwrap(), "STEAM_1:1:422699980");
    }

    #[test]
    fn to_steam_id_3() {
        let id = SteamId(76561198805665689);
        assert_eq!(id.to_steam_id_3().unwrap(), "[U:1:845399961]");
    }
}
