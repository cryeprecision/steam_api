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

fn to_symbol(index: u8) -> Option<u8> {
    match index {
        0..=7 => Some(b'A' + index),        // [A, H]
        8..=12 => Some(b'A' + index + 1),   // [J, N]
        13..=23 => Some(b'A' + index + 2),  // [P, Z]
        24..=31 => Some(b'2' + index - 24), // [2, 9]
        _ => None,
    }
}

fn from_symbol(sym: u8) -> Option<u8> {
    match sym {
        b'A'..=b'H' => Some(sym - b'A'),      // [0, 7]
        b'J'..=b'N' => Some(sym - b'A' - 1),  // [8, 12]
        b'P'..=b'Z' => Some(sym - b'A' - 2),  // [13, 23]
        b'2'..=b'9' => Some(sym - b'2' + 24), // [24, 31]
        _ => None,
    }
}

fn base32_encode(num: u64) -> Option<[u8; 15]> {
    let mut chunks = ChunksU5(num.swap_bytes());
    let mut enc_buf = [0u8; ChunksU5::MAX_CHUNKS + 2];

    {
        enc_buf[00] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[01] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[02] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[03] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[04] = b'-';
        enc_buf[05] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[06] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[07] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[08] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[09] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[10] = b'-';
        enc_buf[11] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[12] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[13] = to_symbol(chunks.next().unwrap_or(0))?;
        enc_buf[14] = to_symbol(chunks.next().unwrap_or(0))?;
    }

    Some(enc_buf)
}

fn base32_decode(code: &[u8]) -> Option<u64> {
    if code.len() != ChunksU5::MAX_CHUNKS + 2 {
        return None;
    }

    let mut result = 0u64;
    let mut dec_buf = [0u8; ChunksU5::MAX_CHUNKS];

    {
        let mut src_iter = code.iter().cloned();
        dec_buf[00] = src_iter.next().and_then(from_symbol)?;
        dec_buf[01] = src_iter.next().and_then(from_symbol)?;
        dec_buf[02] = src_iter.next().and_then(from_symbol)?;
        dec_buf[03] = src_iter.next().and_then(from_symbol)?;
        let _ = src_iter.next(); // skip '-'
        dec_buf[04] = src_iter.next().and_then(from_symbol)?;
        dec_buf[05] = src_iter.next().and_then(from_symbol)?;
        dec_buf[06] = src_iter.next().and_then(from_symbol)?;
        dec_buf[07] = src_iter.next().and_then(from_symbol)?;
        dec_buf[08] = src_iter.next().and_then(from_symbol)?;
        let _ = src_iter.next(); // skip '-'
        dec_buf[09] = src_iter.next().and_then(from_symbol)?;
        dec_buf[10] = src_iter.next().and_then(from_symbol)?;
        dec_buf[11] = src_iter.next().and_then(from_symbol)?;
        dec_buf[12] = src_iter.next().and_then(from_symbol)?;
    }

    for i in 0..ChunksU5::MAX_CHUNKS {
        let shift = (ChunksU5::CHUNK_BITS * i as u32) as u64;
        result |= (dec_buf[i] as u64) << shift;
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
        let hash = self.hash();
        let mut r = 0u64;
        let mut chunks = ChunksU4(self.0);
        for i in 0..8 {
            let a = (r << 4) as u32 | chunks.next().unwrap_or(0) as u32;
            r = u64::from(U32Pair(a, (r >> 28) as u32));
            r = u64::from(U32Pair((a << 1) | ((hash >> i) & 1), (r >> 31) as u32));
        }
        let bytes = base32_encode(r)?;
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
        (&mut buf[..5]).copy_from_slice(b"AAAA-");
        (&mut buf[5..]).copy_from_slice(code);

        let decoded = base32_decode(&buf)?;
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
