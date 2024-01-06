// https://www.ietf.org/rfc/rfc1950.txt

use std::io::{BufRead, Read};

#[derive(Debug)]
pub enum Error {
    UnsupportedCompressionMethod,
    DisallowedWindowBits { cinfo: u8 },
    UnsupportedZlibDict,
    InvalidHeaderChecksum,
    IOErr(std::io::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

pub struct ZlibHeader {
    pub window_size: usize,
    pub flevel: u8,
    pub dictid: Option<u32>,
}

impl ZlibHeader {
    pub fn parse(src: &mut impl Read) -> Result<Self, Error> {
        let mut header_buf = [0u8; 2];
        src.read_exact(&mut header_buf).map_err(Error::IOErr)?;
        let cmf = header_buf[0];
        let flg = header_buf[1];
        let cm = cmf & 0b1111;
        let cinfo = (cmf >> 4) & 0b1111;
        let _fcheck = flg & 0b11111;
        let fdict = (flg >> 5) & 1;
        let flevel = (flg >> 6) & 0b11;
        if cm != 8 {
            return Err(Error::UnsupportedCompressionMethod);
        }
        // cm == 8: "deflate" with window size up to 32K
        if cinfo > 7 {
            return Err(Error::DisallowedWindowBits { cinfo });
        }
        if (((cmf as u16) << 8) + (flg as u16)) % 31 != 0 {
            return Err(Error::InvalidHeaderChecksum);
        }
        let dictid = if fdict == 1 {
            let mut dictid_buf = [0u8; 4];
            src.read_exact(&mut dictid_buf).map_err(Error::IOErr)?;
            let mut v = 0u32;
            for b in dictid_buf.iter() {
                v = (v << 8) | (*b as u32);
            }
            Some(v)
        } else {
            None
        };
        if dictid.is_some() {
            return Err(Error::UnsupportedZlibDict);
        }
        Ok(Self {
            window_size: 1usize << (cinfo + 8),
            flevel,
            dictid,
        })
    }
}

pub fn decode_zlib(src: impl BufRead) -> impl Read {
    flate2::bufread::ZlibDecoder::new(src)
}
