use byteorder::{ByteOrder, LittleEndian};

use crate::bit_chunks::{BitChunks, ChunksU4, ChunksU5};
use crate::steam_id::SteamId;

struct U32Pair(u32, u32);

impl From<U32Pair> for u64 {
    fn from(U32Pair(low, high): U32Pair) -> Self {
        (u64::from(high) << 32) | u64::from(low)
    }
}
impl From<u64> for U32Pair {
    fn from(num: u64) -> Self {
        U32Pair(num as u32, (num >> 32) as u32)
    }
}

fn to_symbol(index: u8) -> Option<char> {
    match index {
        0..=7 => Some((b'A' + index) as char),        // [A, H]
        8..=12 => Some((b'A' + index + 1) as char),   // [J, N]
        13..=23 => Some((b'A' + index + 2) as char),  // [P, Z]
        24..=31 => Some((b'2' + index - 24) as char), // [2, 9]
        _ => None,
    }
}

#[cfg(test)]
fn from_symbol(sym: char) -> Option<u8> {
    match sym {
        'A'..='H' => Some(sym as u8 - b'A'),      // [0, 7]
        'J'..='N' => Some(sym as u8 - b'A' - 1),  // [8, 12]
        'P'..='Z' => Some(sym as u8 - b'A' - 2),  // [13, 23]
        '2'..='9' => Some(sym as u8 - b'2' + 24), // [24, 31]
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

impl SteamId {
    fn hash(self) -> u32 {
        let acc_nr = self.acc_nr() as u32;
        let strange = acc_nr as u64 | 0x4353_474F_0000_0000;
        let digest = md5::compute(strange.to_le_bytes());
        LittleEndian::read_u32(&digest.0)
    }
    pub fn to_friend_code(self) -> String {
        let hash = self.hash();
        let mut r = 0u64;
        let mut chunks = ChunksU4(self.0);
        for i in 0..8 {
            let a = (r << 4) as u32 | u32::from(chunks.next().unwrap_or(0));
            r = u64::from(U32Pair(a, (r >> 28) as u32));
            r = u64::from(U32Pair((a << 1) | ((hash >> i) & 1), (r >> 31) as u32));
        }
        let mut enc = base32_encode(r);
        if enc.starts_with("AAAA-") {
            enc.drain(0.."AAAA-".len());
        };
        enc
    }
}

#[cfg(test)]
mod tests {
    use super::{from_symbol, to_symbol, SteamId};

    #[test]
    fn to_friend_code_works() {
        let id = SteamId(76561197960287930);
        assert_eq!(id.to_friend_code(), "SUCVS-FADA");
    }

    #[test]
    fn from_symbol_offsets() {
        assert_eq!(0, from_symbol('A').unwrap());
        assert_eq!(7, from_symbol('H').unwrap());
        // Skipping `I`
        assert_eq!(8, from_symbol('J').unwrap());
        assert_eq!(12, from_symbol('N').unwrap());
        // Skipping `O`
        assert_eq!(13, from_symbol('P').unwrap());
        assert_eq!(23, from_symbol('Z').unwrap());
        // Skipping `0` and `1`
        assert_eq!(24, from_symbol('2').unwrap());
        assert_eq!(31, from_symbol('9').unwrap());
    }

    #[test]
    fn from_to_symbol_works() {
        let valid = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        valid.chars().enumerate().for_each(|(i, c)| {
            assert_eq!(i, from_symbol(c).unwrap() as usize);
            assert_eq!(to_symbol(i as u8).unwrap(), c);
        });

        for c in "I1O0".chars() {
            assert_eq!(from_symbol(c), None)
        }

        assert_eq!(to_symbol(32), None);
    }
}
