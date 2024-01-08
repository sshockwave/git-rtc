// https://git-scm.com/docs/git-hash-object

use serde::{Deserialize, Serialize};

pub enum Type {
    SHA1,
}

pub enum Hash {
    SHA1([u8; 20]),
}

pub trait Sampled: Serialize + for<'de> Deserialize<'de> {
    type Sample: Eq;
    type Full: Eq;
    fn samples(&self) -> &[Self::Sample];
    fn full(&self) -> &Self::Full;
    fn len(&self) -> u64;
    fn interval(&self) -> u64;
}

pub trait Sample {
    type Result: Sampled;
    fn new(interval: u64);
    fn write(&self, data: &[u8]);
    fn end(self) -> Self::Result;
}

mod sha1 {
    use sha1::digest::{
        block_buffer::{BlockBuffer, Eager},
        crypto_common::BlockSizeUser,
    };

    extern crate sha1;

    #[derive(Clone)]
    struct Buffer {
        register: [u32; 5],
        buffer: BlockBuffer<<sha1::Sha1Core as BlockSizeUser>::BlockSize, Eager>,
        len: u64,
    }

    impl Default for Buffer {
        fn default() -> Self {
            Self {
                register: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                buffer: Default::default(),
                len: 0,
            }
        }
    }

    impl Buffer {
        pub fn write(&mut self, buf: &[u8]) {
            self.len += buf.len() as u64;
            self.buffer
                .digest_blocks(buf, |b| sha1::compress(&mut self.register, b));
        }
        pub fn state(&self) -> State {
            assert!(self.buffer.get_pos() == 0);
            let mut out = [0u8; 20];
            for (chunk, v) in out.chunks_exact_mut(4).zip(self.register.iter()) {
                chunk.copy_from_slice(&v.to_be_bytes());
            }
            State {
                data: out,
                len: self.len,
            }
        }
        pub fn from_state(state: State) -> Self {
            let mut register = [0u32; 5];
            for (v, chunk) in register.iter_mut().zip(state.data.chunks_exact(4)) {
                *v = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
            Self {
                register,
                buffer: Default::default(),
                len: state.len,
            }
        }
        pub fn end(mut self) -> [u8; 20] {
            let bit_len = (self.buffer.get_pos() as u64 + self.len) * 8;
            self.buffer.len64_padding_be(bit_len, |b| {
                sha1::compress(&mut self.register, core::slice::from_ref(b))
            });
            self.state().data
        }
    }

    #[derive(Clone)]
    struct State {
        data: [u8; 20],
        len: u64,
    }

    impl From<State> for super::Hash {
        fn from(state: State) -> Self {
            Self::SHA1(state.data)
        }
    }

    impl State {
        pub fn from_hash(hash: [u8; 20], len: u64) -> Self {
            Self { data: hash, len }
        }
        pub fn len(&self) -> u64 {
            self.len
        }
    }

}
