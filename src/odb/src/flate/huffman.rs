use crate::flate::BitBuffer;

pub trait Table {
    // bit_buf might contain extra 0s at the end,
    // so it is required to check that
    // the returned length can be covered by the bit_buf
    fn lookup(&self, bit_buf: super::BitBuffer) -> Option<(u16, usize)>; // (symbol, length)
}

struct FastTable<const N: usize> {
    // lower 9 bits are the symbol
    // higher 7 bits are the length of the encoded symbol (15 max)
    table: [u16; N],
}

impl<const N: usize> Table for FastTable<N> {
    fn lookup(&self, bit_buf: super::BitBuffer) -> Option<(u16, usize)> {
        assert!(N & (N - 1) == 0); // N is a power of 2
        let result = self.table[bit_buf & (N - 1)];
        if result > 0 {
            Some((result & 0x1ff, (result >> 9) as usize))
        } else {
            None
        }
    }
}

pub struct FixedLitTable;
impl Table for FixedLitTable {
    fn lookup(&self, bit_buf: BitBuffer) -> Option<(u16, usize)> {
        const fn put_high(bits: u16, len: u32) -> BitBuffer {
            (bits as BitBuffer) << (BitBuffer::BITS - len)
        }
        const fn put_low(len: u32) -> BitBuffer {
            (1 << (BitBuffer::BITS - len)) - 1
        }
        const fn get_high(bits: BitBuffer, len: u32) -> u16 {
            (bits >> (BitBuffer::BITS - len)) as u16
        }
        const L1: BitBuffer = put_high(0b00110000, 8);
        const R1: BitBuffer = put_high(0b10111111, 8) | put_low(8);
        const L2: BitBuffer = put_high(0b110010000, 9);
        const R2: BitBuffer = put_high(0b111111111, 9) | put_low(9);
        const L3: BitBuffer = put_high(0b0000000, 7);
        const R3: BitBuffer = put_high(0b0010111, 7) | put_low(7);
        const L4: BitBuffer = put_high(0b11000000, 8);
        const R4: BitBuffer = put_high(0b11000111, 8) | put_low(8);
        const _: () = assert!(R2 == BitBuffer::MAX);
        Some(match bit_buf.reverse_bits() {
            L1..=R1 => (get_high(bit_buf, 8) - 0b00110000, 8),
            L2.. => (get_high(bit_buf, 9) - 0b11001000 + 144, 9),
            L3..=R3 => (get_high(bit_buf, 7) - 0b0000000 + 256, 7),
            L4..=R4 => (get_high(bit_buf, 8) - 0b11000000 + 280, 8),
        })
    }
}

pub struct FixedDistTable;
impl Table for FixedDistTable {
    fn lookup(&self, bit_buf: super::BitBuffer) -> Option<(u16, usize)> {
        let val = bit_buf.reverse_bits() >> (BitBuffer::BITS - 5);
        if val <= 29 {
            Some((val as u16, 5))
        } else {
            None
        }
    }
}

const MAX_CODE_LEN: u8 = 15;
struct FullTable {
    tree: Vec<i16>,
}
impl FullTable {
    fn from_code_len(code_len: &[u8]) -> Self {
        let mut code_len_cnt = [0; MAX_CODE_LEN as usize];
        for &len in code_len.iter() {
            code_len_cnt[len as usize] += 1;
        }
        let mut next_code = [0; MAX_CODE_LEN as usize];
        let mut code: u16 = 0;
        code_len_cnt[0] = 0;
        let mut tree_size: usize = 1; // root
        let mut last_len = 0;
        for len in 1..=MAX_CODE_LEN {
            let cnt = code_len_cnt[len as usize];
            if cnt == 0 {
                continue;
            }
            fn tree_delta(mut l: usize, mut r: usize) -> usize {
                let mut sum = 0;
                while l != r {
                    sum += r - l;
                    l >>= 1;
                    r >>= 1;
                }
                sum
            }
            if code > 0 {
                tree_size += ((code - 1) ^ code).count_ones() as usize;
                // equivalent to tree_delta(code as usize - 1, code as usize);
            }
            let delta_len = len - last_len;
            last_len = len;
            tree_size += delta_len as usize;
            code <<= delta_len;
            tree_size += tree_delta(code as usize, (code + cnt - 1) as usize) as usize;
            next_code[len as usize] = code;
            code += cnt;
        }
        let mut tree = Vec::new();
        tree.resize(tree_size, 0);
        todo!();
        Self { tree }
    }
}

struct CachedTable {/* todo */}
