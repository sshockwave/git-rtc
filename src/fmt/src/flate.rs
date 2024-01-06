// https://www.ietf.org/rfc/rfc1951.txt

use std::io::{BufRead, Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum Error {
    ReservedBtype { block_position: u64 },
    OnesComplementMismatch { block_position: u64 },
    IOErr(std::io::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

pub struct ParsedDeflate<R: Read + Seek> {
    src: R,
    offsets: Vec<(u64, usize)>, // src_offset, out_offset
    more: bool,
}

impl<R: Read + Seek> ParsedDeflate<R> {
    pub fn parse(mut src: R) -> Result<Self, Error> {
        use Error::IOErr;
        let mut offsets = Vec::new();
        let mut more = true;
        let mut out_pos = 0usize;
        loop {
            let src_pos = src.stream_position().map_err(IOErr)?;
            offsets.push((src_pos, out_pos));
            if !more {
                break;
            }
            let mut header = [0u8; 1];
            src.read_exact(&mut header).map_err(IOErr)?;
            let bfinal = header[0] & 1;
            let btype = (header[0] >> 1) & 3;
            match btype {
                // no compression
                0b00 => {
                    let mut len_buf = [0u8; 4];
                    src.read_exact(&mut len_buf).map_err(IOErr)?;
                    more = bfinal == 0;
                    let len = len_buf[0] as u16 | ((len_buf[1] as u16) << 8u16);
                    let nlen = len_buf[2] as u16 | ((len_buf[3] as u16) << 8u16);
                    if len != !nlen {
                        return Err(Error::OnesComplementMismatch {
                            block_position: src_pos,
                        });
                    }
                    src.seek(SeekFrom::Current(len as i64)).map_err(IOErr)?;
                    out_pos += len as usize;
                }
                // compressed
                0b01 | 0b10 => {
                    src.seek(SeekFrom::Start(src_pos)).map_err(IOErr)?; // rewind
                    break;
                }
                // reserved
                0b11 => {
                    return Err(Error::ReservedBtype {
                        block_position: src_pos,
                    });
                }
                4u8..=u8::MAX => unreachable!(),
            }
        }
        Ok(Self { src, offsets, more })
    }

    pub fn to_seek_reader(self) -> Result<impl Read + Seek, Self> {
        if self.more {
            Ok(SeekReadDeflate {
                parsed: self,
                block_idx: 0,
                out_pos: 0,
            })
        } else {
            Err(self)
        }
    }

    pub fn into_inner(self) -> R {
        self.src
    }
}

struct SeekReadDeflate<R: Read + Seek> {
    parsed: ParsedDeflate<R>,
    block_idx: usize,
    out_pos: usize,
}

impl<R: Read + Seek> Read for SeekReadDeflate<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl<R: Read + Seek> Seek for SeekReadDeflate<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

pub fn decode_deflate(src: impl BufRead) -> impl Read {
    flate2::bufread::DeflateDecoder::new(src)
}
