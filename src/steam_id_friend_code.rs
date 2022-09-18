use crate::bit_chunks::{BitChunks, ChunksU4, ChunksU5};
use crate::steam_id::SteamId;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

struct U32Pair(u32, u32);

impl From<U32Pair> for u64 {
    fn from(U32Pair(low, high): U32Pair) -> Self {
        ((high as u64) << 32) | (low as u64)
    }
}
impl From<u64> for U32Pair {
    fn from(num: u64) -> Self {
        U32Pair(num as u32, (num >> 32) as u32)
    }
}

fn to_symbol(index: u8) -> Option<char> {
    match index {
        0..=7 => Some(('A' as u8 + index) as char),      // [A, H]
        8..=12 => Some(('A' as u8 + index + 1) as char), // [J, N]
        13..=23 => Some(('A' as u8 + index + 2) as char), // [P, Z]
        24..=31 => Some(('2' as u8 + index - 24) as char), // [2, 9]
        _ => None,
    }
}

fn _from_symbol(sym: char) -> Option<u8> {
    match sym {
        'A'..='H' => Some(sym as u8 - 'A' as u8),      // [0, 7]
        'J'..='N' => Some(sym as u8 - 'A' as u8 - 1),  // [8, 12]
        'P'..='Z' => Some(sym as u8 - 'A' as u8 - 2),  // [13, 23]
        '2'..='9' => Some(sym as u8 - '2' as u8 + 24), // [24, 31]
        _ => None,
    }
}

fn base32_encode(num: u64) -> String {
    let mut chunks = ChunksU5(num.swap_bytes());
    let mut buf = String::with_capacity(ChunksU5::MAX_CHUNKS + 2);
    for i in 0..ChunksU5::MAX_CHUNKS {
        if i == 4 || i == 9 {
            buf.push('-');
        }
        buf.push(to_symbol(chunks.next().unwrap_or(0)).unwrap());
    }
    buf
}

fn _base32_decode(_str: &str) -> SteamId {
    todo!();
}

impl SteamId {
    fn hash(&self) -> u32 {
        let acc_nr = self.acc_nr() as u32;
        let strange = acc_nr as u64 | 0x4353474F00000000;
        let digest = md5::compute(&strange.to_le_bytes());
        LittleEndian::read_u32(&digest.0)
    }
    pub fn to_friend_code(&self) -> String {
        let hash = self.hash();
        let mut r = 0u64;
        let mut chunks = ChunksU4(self.0);
        for i in 0..8 {
            let a = (r << 4) as u32 | chunks.next().unwrap_or(0) as u32;
            r = u64::from(U32Pair(a, (r >> 28) as u32));
            r = u64::from(U32Pair((a << 1) | ((hash >> i) & 1), (r >> 31) as u32));
        }
        let mut enc = base32_encode(r);
        if enc.starts_with("AAAA-") {
            let _ = enc.drain(0.."AAAA-".len());
        };
        enc
    }
    pub fn from_friend_code(_friend_code: &str) -> Option<SteamId> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::{_from_symbol, to_symbol, SteamId};

    #[test]
    fn to_friend_code_works() {
        let id = SteamId(76561197960287930);
        assert_eq!(id.hash(), 0x890C9498);
        assert_eq!(id.to_friend_code(), "SUCVS-FADA");
    }

    #[test]
    fn from_to_symbol_works() {
        let valid = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        valid.chars().enumerate().for_each(|(i, c)| {
            assert_eq!(i, _from_symbol(c).unwrap() as usize);
            assert_eq!(to_symbol(i as u8).unwrap(), c);
        });

        for c in "I1O0".chars() {
            assert_eq!(_from_symbol(c), None)
        }

        assert_eq!(to_symbol(32), None);
    }
}
