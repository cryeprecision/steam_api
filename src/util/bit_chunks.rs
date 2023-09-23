pub trait BitChunks<T> {
    const CHUNK_BITS: u32;
    const MAX_CHUNKS: usize;
    const CHUNK_MASK: T;
}

macro_rules! impl_bit_chunks {
    ($SelfT:ident, $ActualT:ty, $ItemT:ty, $BITS:expr) => {
        pub struct $SelfT(pub $ActualT);
        impl BitChunks<$ActualT> for $SelfT {
            const CHUNK_BITS: u32 = $BITS;
            const MAX_CHUNKS: usize = <$ActualT>::BITS.div_ceil(Self::CHUNK_BITS) as usize;
            const CHUNK_MASK: $ActualT = (1 << Self::CHUNK_BITS) - 1;
        }
        impl Iterator for $SelfT {
            type Item = $ItemT;
            fn next(&mut self) -> Option<Self::Item> {
                let out = match self.0 {
                    0 => None,
                    _ => Some((self.0 & Self::CHUNK_MASK) as Self::Item),
                };
                self.0 >>= Self::CHUNK_BITS;
                out
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                let bits_left = (<$ActualT>::BITS - self.0.leading_zeros()) as usize;
                let len = bits_left.div_ceil(Self::CHUNK_BITS as usize);
                (len, Some(len))
            }
        }
        impl ExactSizeIterator for $SelfT {}
    };
}

impl_bit_chunks!(ChunksU4, u64, u8, 4);
impl_bit_chunks!(ChunksU5, u64, u8, 5);
impl_bit_chunks!(ChunksU6, u64, u8, 6);

mod test {
    macro_rules! test_bit_chunks {
        ($func:ident, $SelfT:ident, $ItemT:ty, $ActualT:ty, $MAX_CHUNKS:expr, $value:expr, $($expec:expr),+) => {
            #[test]
            fn $func() {
                use super::{BitChunks, $SelfT};
                assert_eq!(<$SelfT>::MAX_CHUNKS, $MAX_CHUNKS);
                let arr: &[$ItemT] = &[$($expec),+];
                for (chunk, exp) in $SelfT($value).zip(arr.iter()) {
                    assert_eq!(*exp, chunk);
                }
                assert_eq!($SelfT($value).skip(arr.len()).next(), None);
            }
        };
    }

    test_bit_chunks!(u4_works, ChunksU4, u8, u64, 16, 0xFFFF, 0x0F, 0x0F, 0x0F, 0x0F);
    test_bit_chunks!(u5_works, ChunksU5, u8, u64, 13, 0xFFFF, 0x1F, 0x1F, 0x1F, 0x01);
    test_bit_chunks!(u6_works, ChunksU6, u8, u64, 11, 0xFFFF, 0x3F, 0x3F, 0x0F);
}
