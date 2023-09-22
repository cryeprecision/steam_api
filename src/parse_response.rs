pub trait ParseResponse<T>: Sized {
    type Error;
    fn parse_response(value: T) -> std::result::Result<Self, Self::Error>;
}

pub trait ParseJsonResponse: Sized {
    /// TODO
    type Error;
    /// TODO
    type Output;

    /// TODO
    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error>;
}
