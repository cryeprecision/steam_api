use serde::Serialize;

pub trait ParseResponse<T>: Sized {
    type Error;
    fn parse_response(value: T) -> std::result::Result<Self, Self::Error>;
}

pub trait ParseJsonResponse {
    /// TODO
    type Error;
    /// TODO
    type Output;

    /// TODO
    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error>;
}

pub trait SteamQuerySingle {
    /// TODO
    type Output: Serialize;

    /// TODO
    fn to_query_single(&self) -> Self::Output;
}

pub trait SteamQueryMultiple {
    /// TODO
    type Output: Serialize;

    /// TODO
    fn to_query_multiple(&self) -> Self::Output;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        // TODO
    }
}
