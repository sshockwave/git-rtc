use std::io::{self, BufRead, Read};

// https://www.ietf.org/rfc/rfc1950.txt
pub struct ZlibHeader {
    pub window_size: usize,
    pub flevel: u8,
    pub dictid: Option<u32>,
}

impl ZlibHeader {
    pub fn parse(src: &mut impl Read) -> io::Result<Self> {
        let mut header_buf = [0u8; 2];
        src.read_exact(&mut header_buf)?;
        let cmf = header_buf[0];
        let flg = header_buf[1];
        let cm = cmf & 0b1111;
        let cinfo = (cmf >> 4) & 0b1111;
        let _fcheck = flg & 0b11111;
        let fdict = (flg >> 5) & 1;
        let flevel = (flg >> 6) & 0b11;
        // cm == 8: "deflate" with window size up to 32K
        if cm != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported compression method",
            ));
        }
        if cinfo > 7 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid window size {}", cinfo),
            ));
        }
        if (((cmf as u16) << 8) + (flg as u16)) % 31 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid header checksum",
            ));
        }
        let dictid = if fdict == 1 {
            let mut dictid_buf = [0u8; 4];
            src.read_exact(&mut dictid_buf)?;
            let mut v = 0u32;
            for b in dictid_buf.iter() {
                v = (v << 8) | (*b as u32);
            }
            Some(v)
        } else {
            None
        };
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
