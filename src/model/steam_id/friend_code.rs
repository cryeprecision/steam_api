use byteorder::{ByteOrder, LittleEndian};

use crate::model::SteamId;
use crate::util::bit_chunks::{BitChunks, ChunksU4, ChunksU5};

const fn u32x2_to_u64(low: u32, high: u32) -> u64 {
    ((high as u64) << 32) | (low as u64)
}

const fn to_symbol(index: u8) -> Option<u8> {
    match index {
        0..=7 => Some(b'A' + index),        // [A, H]
        8..=12 => Some(b'A' + index + 1),   // [J, N]
        13..=23 => Some(b'A' + index + 2),  // [P, Z]
        24..=31 => Some(b'2' + index - 24), // [2, 9]
        _ => None,
    }
}

const fn from_symbol(sym: u8) -> Option<u8> {
    match sym {
        b'A'..=b'H' => Some(sym - b'A'),      // [0, 7]
        b'J'..=b'N' => Some(sym - b'A' - 1),  // [8, 12]
        b'P'..=b'Z' => Some(sym - b'A' - 2),  // [13, 23]
        b'2'..=b'9' => Some(sym - b'2' + 24), // [24, 31]
        _ => None,
    }
}

fn base32_encode_u64(num: u64) -> Option<[u8; 15]> {
    let mut chunks = ChunksU5(num.swap_bytes());
    let mut enc_buf = [0u8; ChunksU5::MAX_CHUNKS + 2];

    for (i, enc) in enc_buf.iter_mut().enumerate() {
        if i == 4 || i == 10 {
            *enc = b'-';
        } else {
            *enc = to_symbol(chunks.next().unwrap_or(0))?;
        }
    }

    Some(enc_buf)
}

fn base32_decode_u64(code: [u8; 15]) -> Option<u64> {
    let mut result = 0u64;
    let mut dec_buf = [0u8; ChunksU5::MAX_CHUNKS];

    let mut src_iter = code.iter().cloned();
    for (i, dec) in dec_buf.iter_mut().enumerate() {
        if i == 4 || i == 10 {
            let _ = src_iter.next(); // skip '-'
        }
        *dec = src_iter.next().and_then(from_symbol)?;
    }

    for (i, dec) in dec_buf.iter().cloned().enumerate() {
        let shift = (ChunksU5::CHUNK_BITS * i as u32) as u64;
        result |= (dec as u64) << shift;
    }

    Some(result.to_be())
}

impl SteamId {
    fn hash(self) -> u32 {
        /// <https://www.unknowncheats.me/forum/counterstrike-global-offensive/453555-de-encoding-cs-friend-codes.html>
        const FILLER: u64 = u64::from_be_bytes([b'C', b'S', b'G', b'O', 0, 0, 0, 0]);
        const FILLER_MASK: u64 = 0x0000_0000_FFFF_FFFF;

        let bytes = (self.as_u64() & FILLER_MASK) | FILLER;
        let digest = md5::compute(bytes.to_le_bytes());
        LittleEndian::read_u32(&digest.0)
    }

    pub fn to_friend_code(self) -> Option<String> {
        let mut chunks = ChunksU4(self.0);

        let mut hash = self.hash();
        let mut r = 0u64;
        for _ in 0..8 {
            let a = (r << 4) as u32 | (chunks.next().unwrap_or(0) as u32);
            r = u32x2_to_u64(a, (r >> 28) as u32);
            r = u32x2_to_u64((a << 1) | (hash & 1), (r >> 31) as u32);
            hash >>= 1;
        }

        let bytes = base32_encode_u64(r)?;
        std::str::from_utf8(bytes.strip_prefix(b"AAAA-")?)
            .ok()
            .map(|s| s.to_string())
    }

    pub fn from_friend_code(code: &str) -> Option<SteamId> {
        const DEFAULT_STEAM_ID: u64 = 0x0110_0001_0000_0000;

        let code = code.as_bytes();
        if code.len() != "SUCVS-FADA".len() {
            return None;
        }

        let mut buf = [0u8; ChunksU5::MAX_CHUNKS + 2];
        buf[..5].copy_from_slice(b"AAAA-");
        buf[5..].copy_from_slice(code);

        let decoded = base32_decode_u64(buf)?;
        let mut chunks = ChunksU5(decoded);

        let mut steam_id = 0u64;
        for _ in 0..8 {
            let chunk = chunks.next().unwrap_or(0);
            steam_id <<= 4;
            steam_id |= ((chunk & 0b0001_1110) >> 1) as u64;
        }
        steam_id |= DEFAULT_STEAM_ID;

        Some(SteamId(steam_id))
    }
}

#[cfg(test)]
mod tests {
    use super::{from_symbol, to_symbol, SteamId};

    #[test]
    fn to_friend_code_works() {
        let id = SteamId(76561197960287930);
        assert_eq!(id.to_friend_code(), Some("SUCVS-FADA".to_string()));

        let id = SteamId(76561199006131828);
        assert_eq!(id.to_friend_code(), Some("SBPVY-4MQJ".to_string()));
    }

    #[test]
    fn from_friend_code_works() {
        let code = "SUCVS-FADA";
        assert_eq!(
            SteamId::from_friend_code(code),
            Some(SteamId(76561197960287930))
        );

        let code = "SBPVY-4MQJ";
        assert_eq!(
            SteamId::from_friend_code(code),
            Some(SteamId(76561199006131828))
        );
    }

    #[test]
    fn from_symbol_offsets() {
        assert_eq!(0, from_symbol(b'A').unwrap());
        assert_eq!(7, from_symbol(b'H').unwrap());
        // Skipping `I`
        assert_eq!(8, from_symbol(b'J').unwrap());
        assert_eq!(12, from_symbol(b'N').unwrap());
        // Skipping `O`
        assert_eq!(13, from_symbol(b'P').unwrap());
        assert_eq!(23, from_symbol(b'Z').unwrap());
        // Skipping `0` and `1`
        assert_eq!(24, from_symbol(b'2').unwrap());
        assert_eq!(31, from_symbol(b'9').unwrap());
    }

    #[test]
    fn from_to_symbol_works() {
        let valid = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let iter = valid.iter().cloned().enumerate();
        iter.for_each(|(i, c)| {
            assert_eq!(i, from_symbol(c).unwrap() as usize);
            assert_eq!(to_symbol(i as u8).unwrap(), c);
        });

        for c in b"I1O0".iter().cloned() {
            assert_eq!(from_symbol(c), None)
        }

        assert_eq!(to_symbol(32), None);
    }
}
