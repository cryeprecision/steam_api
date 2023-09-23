use super::EnumError;

/// <https://developer.valvesoftware.com/wiki/SteamID#Types_of_Steam_Accounts>
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

impl TryFrom<u64> for AccountType {
    type Error = EnumError<u64>;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AccountType::Invalid),
            1 => Ok(AccountType::Individual),
            2 => Ok(AccountType::Multiseat),
            3 => Ok(AccountType::GameServer),
            4 => Ok(AccountType::AnonGameServer),
            5 => Ok(AccountType::Pending),
            6 => Ok(AccountType::ContentServer),
            7 => Ok(AccountType::Clan),
            8 => Ok(AccountType::Chat),
            9 => Ok(AccountType::SuperSeeder),
            10 => Ok(AccountType::AnonUser),
            _ => Err(EnumError::Unknown(value)),
        }
    }
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
