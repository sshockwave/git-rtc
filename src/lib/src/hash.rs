// https://git-scm.com/docs/git-hash-object

use ::{
    core::fmt,
    serde::{de, Deserialize, Serialize},
};

pub trait Hasher: Default {
    type Sample: Serialize + for<'de> Deserialize<'de> + Eq + Clone;
    type Result: Serialize + for<'de> Deserialize<'de> + Eq + Clone + ToString;
    fn sample(&self) -> Self::Sample;
    fn from_sample(sample: Self::Sample, len: u64) -> Self;
    fn end(self) -> Self::Result;
    fn write(&mut self, data: &[u8]);
}

mod sha1 {
    use ::sha1::{
        compress,
        digest::{
            block_buffer::{BlockBuffer, Eager},
            crypto_common::BlockSizeUser,
        },
        Sha1Core,
    };

    pub struct Hasher {
        register: [u32; 5],
        buffer: BlockBuffer<<Sha1Core as BlockSizeUser>::BlockSize, Eager>,
        len: u64,
    }

    impl Default for Hasher {
        fn default() -> Self {
            Self {
                register: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                buffer: Default::default(),
                len: 0,
            }
        }
    }

    impl super::Hasher for Hasher {
        type Result = super::HashBytes<20>;
        type Sample = super::HashBytes<20>;
        fn write(&mut self, data: &[u8]) {
            self.len += data.len() as u64;
            self.buffer
                .digest_blocks(data, |b| compress(&mut self.register, b));
        }
        fn sample(&self) -> Self::Sample {
            assert!(self.buffer.get_pos() == 0);
            let mut out = [0u8; 20];
            for (chunk, v) in out.chunks_exact_mut(4).zip(self.register.iter()) {
                chunk.copy_from_slice(&v.to_be_bytes());
            }
            Self::Sample { data: out }
        }
        fn from_sample(state: Self::Sample, len: u64) -> Self {
            let mut register = [0u32; 5];
            for (v, chunk) in register.iter_mut().zip(state.data.chunks_exact(4)) {
                *v = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
            Self {
                register,
                buffer: Default::default(),
                len: len,
            }
        }
        fn end(mut self) -> Self::Result {
            let bit_len = (self.buffer.get_pos() as u64 + self.len) * 8;
            self.buffer.len64_padding_be(bit_len, |b| {
                compress(&mut self.register, core::slice::from_ref(b))
            });
            self.sample()
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HashBytes<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> ToString for HashBytes<N> {
    fn to_string(&self) -> String {
        ::hex::encode(&self.data)
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for HashBytes<N> {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor<const N: usize>;
        impl<'de, const N: usize> de::Visitor<'de> for Visitor<N> {
            type Value = HashBytes<{ N }>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex string of length 40")
            }
            fn visit_str<E>(self, v: &str) -> ::core::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut data = [0u8; N];
                ::hex::decode_to_slice(v, &mut data).map_err(E::custom)?;
                Ok(HashBytes { data })
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl<const N: usize> serde::Serialize for HashBytes<N> {
    fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
