use std::fmt;
use std::fmt::Write;
use std::str::FromStr;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SteamId(pub u64);

#[derive(Debug, Error)]
pub enum SteamIdError {
    #[error("couldn't parse steam-id")]
    InvalidString(#[from] std::num::ParseIntError),
}
type Result<T> = std::result::Result<T, SteamIdError>;

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for SteamId {
    type Err = SteamIdError;
    fn from_str(s: &str) -> Result<Self> {
        Ok(SteamId(s.parse::<u64>()?))
    }
}

impl From<u64> for SteamId {
    fn from(id: u64) -> Self {
        Self(id)
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
    /// Credit for the formula goes to [`exploringbinary.com`](https://www.exploringbinary.com/number-of-decimal-digits-in-a-binary-integer/)
    pub const MAX_DIGITS_FOR_U64: usize = 20;

    /// [`W = 2 * Z + Y`](https://developer.valvesoftware.com/wiki/SteamID#Steam_ID_as_a_Steam_Community_ID#:~:text=W%3DZ*2%2BY)
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

    pub const fn acc_type(&self) -> Option<AccountType> {
        match (self.0 >> Self::TYPE_SHIFT) & Self::TYPE_MASK {
            0 => Some(AccountType::Invalid),
            1 => Some(AccountType::Individual),
            2 => Some(AccountType::Multiseat),
            3 => Some(AccountType::GameServer),
            4 => Some(AccountType::AnonGameServer),
            5 => Some(AccountType::Pending),
            6 => Some(AccountType::ContentServer),
            7 => Some(AccountType::Clan),
            8 => Some(AccountType::Chat),
            9 => Some(AccountType::SuperSeeder),
            10 => Some(AccountType::AnonUser),
            _ => None,
        }
    }

    pub const fn universe(&self) -> Option<Universe> {
        match (self.0 >> Self::UNIVERSE_SHIFT) & Self::UNIVERSE_MASK {
            0 => Some(Universe::Invalid),
            1 => Some(Universe::Public),
            2 => Some(Universe::Beta),
            3 => Some(Universe::Internal),
            4 => Some(Universe::Dev),
            5 => Some(Universe::Rc),
            _ => None,
        }
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// [`As Represented Textually`](https://developer.valvesoftware.com/wiki/SteamID#As_Represented_Textually)
    pub fn to_steam_id(&self) -> Option<String> {
        let x = self.universe()?.as_u64();
        let y = self.y();
        let z = self.acc_nr();
        let mut buf = String::with_capacity("STEAM_X:X:XXXXXXXXXX".len());
        write!(buf, "STEAM_{}:{}:{}", x, y, z).unwrap();
        Some(buf)
    }

    /// [`Steam ID as a Steam Community ID`](https://developer.valvesoftware.com/wiki/SteamID#Steam_ID_as_a_Steam_Community_ID)
    pub fn to_steam_id_3(&self) -> Option<String> {
        let letter = self.acc_type()?.to_letter()?;
        let w = self.w();
        let mut buf = String::with_capacity("[X:1:XXXXXXXXXX]".len());
        write!(buf, "[{}:1:{}]", letter, w).unwrap();
        Some(buf)
    }
}

/// [`Types of Steam Accounts`](https://developer.valvesoftware.com/wiki/SteamID#Types_of_Steam_Accounts)
#[derive(PartialEq, Eq, Debug)]
pub enum AccountType {
    Invalid,
    Individual,
    Multiseat,
    GameServer,
    AnonGameServer,
    Pending,
    ContentServer,
    Clan,
    Chat,
    SuperSeeder,
    AnonUser,
}

impl AccountType {
    pub const fn to_letter(self) -> Option<char> {
        match self {
            AccountType::Invalid => Some('I'),
            AccountType::Individual => Some('U'),
            AccountType::Multiseat => Some('M'),
            AccountType::GameServer => Some('G'),
            AccountType::AnonGameServer => Some('A'),
            AccountType::Pending => Some('P'),
            AccountType::ContentServer => Some('C'),
            AccountType::Clan => Some('g'),
            AccountType::Chat | AccountType::SuperSeeder => None,
            AccountType::AnonUser => Some('a'),
        }
    }
    pub const fn as_u64(self) -> u64 {
        match self {
            AccountType::Invalid => 0,
            AccountType::Individual => 1,
            AccountType::Multiseat => 2,
            AccountType::GameServer => 3,
            AccountType::AnonGameServer => 4,
            AccountType::Pending => 5,
            AccountType::ContentServer => 6,
            AccountType::Clan => 7,
            AccountType::Chat => 8,
            AccountType::SuperSeeder => 9,
            AccountType::AnonUser => 10,
        }
    }
}

/// [`Universes Available for Steam Accounts`](https://developer.valvesoftware.com/wiki/SteamID#Universes_Available_for_Steam_Accounts)
#[derive(PartialEq, Eq, Debug)]
pub enum Universe {
    Invalid,
    Public,
    Beta,
    Internal,
    Dev,
    Rc,
}

impl Universe {
    pub const fn as_u64(self) -> u64 {
        match self {
            Universe::Invalid => 0,
            Universe::Public => 1,
            Universe::Beta => 2,
            Universe::Internal => 3,
            Universe::Dev => 4,
            Universe::Rc => 5,
        }
    }
}

struct SteamIdVisitor;

impl<'de> Visitor<'de> for SteamIdVisitor {
    type Value = SteamId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a steam id either as a string or unsigned integer")
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SteamId::from(v))
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v: u64 = v
            .parse()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &self))?;
        self.visit_u64(v)
    }
    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_borrowed_str(v.as_str())
    }
}

impl<'de> Deserialize<'de> for SteamId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SteamIdVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::SteamId;

    #[test]
    fn deserialize_steam_ids_str() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_ids: Vec<SteamId>,
        }

        let json = serde_json::json!({
            "steam_ids": ["76561198805665689", "76561197992321696"],
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let mut steam_ids = parsed.steam_ids.into_iter();
        assert_eq!(steam_ids.next(), Some(SteamId(76561198805665689)));
        assert_eq!(steam_ids.next(), Some(SteamId(76561197992321696)));
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
    fn deserialize_steam_id_str() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            steam_id: SteamId,
        }

        let json = serde_json::json!({
            "steam_id": "76561198805665689",
        })
        .to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.steam_id, SteamId(76561198805665689));
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
