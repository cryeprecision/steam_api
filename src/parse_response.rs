pub trait ParseResponse<T>: Sized {
    type Error;
    fn parse_response(value: T) -> std::result::Result<Self, Self::Error>;
}
