use crate::steam_id::SteamId;
use std::borrow::Borrow;
use std::fmt::{Display, Write};

/// Extends iterators that iterate over [`SteamId`]s or [`&SteamId`](SteamId)s
pub trait SteamIdExt: Iterator {
    /// Builds a string by using up the iterator.
    ///
    /// Tries to be efficient, by approximating the size of the resulting
    /// string and initially allocating enough space for the whole thing.
    fn to_steam_id_string<T>(mut self, sep: &str) -> String
    where
        Self: Sized + Iterator<Item = T>,
        T: Borrow<SteamId>,
    {
        let (lower, _) = self.size_hint();
        let cap = lower * SteamId::MAX_DIGITS_FOR_U64 + lower.saturating_sub(1) * sep.len();
        let mut buf = String::with_capacity(cap);
        if let Some(id) = self.next() {
            write!(buf, "{}", id.borrow()).unwrap();
            while let Some(id) = self.next() {
                buf.push_str(sep);
                write!(buf, "{}", id.borrow()).unwrap();
            }
        }
        buf
    }

    /// Builds a string by invoking `f` with each element of the iterator.
    ///
    /// Not nearly as efficient as [`SteamIdExt::to_steam_id_string`] because this function cannot allocate
    /// a large enough string up front, since it isn't known how many chars are needed to display
    /// the result of `f`.
    fn to_steam_id_string_with<T, F, B>(mut self, sep: &str, f: F) -> String
    where
        Self: Sized + Iterator<Item = T>,
        T: Borrow<SteamId>,
        F: Fn(&SteamId) -> B,
        B: Display,
    {
        let mut buf = String::new();
        if let Some(id) = self.next() {
            write!(buf, "{}", f(id.borrow())).unwrap();
            while let Some(id) = self.next() {
                buf.push_str(sep);
                write!(buf, "{}", f(id.borrow())).unwrap();
            }
        }
        buf
    }
}
impl<T: Borrow<SteamId>, I: Iterator<Item = T>> SteamIdExt for I {}

#[cfg(test)]
mod tests {
    use super::{SteamId, SteamIdExt};

    #[test]
    fn to_steam_id_string_works() {
        let vec = vec![SteamId(76561197960287930), SteamId(76561197985607672)];
        assert_eq!(
            vec.iter().to_steam_id_string(", "),
            "76561197960287930, 76561197985607672"
        );
        assert_eq!(
            vec.into_iter().to_steam_id_string(", "),
            "76561197960287930, 76561197985607672"
        );

        let slice = &[SteamId(76561197960287930), SteamId(76561197985607672)];
        assert_eq!(
            slice.iter().to_steam_id_string(", "),
            "76561197960287930, 76561197985607672"
        );
        assert_eq!(
            slice.into_iter().to_steam_id_string(", "),
            "76561197960287930, 76561197985607672"
        );
    }

    #[test]
    fn to_steam_id_string_capacity_works() {
        let slice = &[SteamId(76561197960287930), SteamId(76561197985607672)];
        let result = slice.iter().to_steam_id_string(", ");
        assert_eq!(
            result.capacity(),
            SteamId::MAX_DIGITS_FOR_U64 * 2 + ", ".len()
        );
    }

    #[test]
    fn to_steam_id_string_with_works() {
        // 76561197960287930 => ([U:1:22202], STEAM_1:0:11101)
        // 76561197985607672 => ([U:1:25341944], STEAM_1:0:12670972)
        let slice = &[SteamId(76561197960287930), SteamId(76561197985607672)];
        assert_eq!(
            slice
                .iter()
                .to_steam_id_string_with(", ", |id| id.to_steam_id().unwrap()),
            "STEAM_1:0:11101, STEAM_1:0:12670972"
        );
        assert_eq!(
            slice
                .iter()
                .to_steam_id_string_with(", ", |id| id.to_steam_id_3().unwrap()),
            "[U:1:22202], [U:1:25341944]"
        );
    }
}
