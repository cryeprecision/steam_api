use super::EnumError;

/// <https://developer.valvesoftware.com/wiki/SteamID#Universes_Available_for_Steam_Accounts>
#[derive(PartialEq, Eq, Debug)]
pub enum Universe {
    Invalid,
    Public,
    Beta,
    Internal,
    Dev,
    Rc,
}

impl TryFrom<u64> for Universe {
    type Error = EnumError<u64>;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Universe::Invalid),
            1 => Ok(Universe::Public),
            2 => Ok(Universe::Beta),
            3 => Ok(Universe::Internal),
            4 => Ok(Universe::Dev),
            5 => Ok(Universe::Rc),
            _ => Err(EnumError::Unknown(value)),
        }
    }
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
