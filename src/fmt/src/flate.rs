// https://www.ietf.org/rfc/rfc1951.txt

use std::io::{self, Read, Seek, SeekFrom};

pub struct ParsedDeflate<R: Read + Seek> {
    src: R,
    block_end: Vec<(u64, u64)>, // src_offset, out_offset of the end of each block
    more: bool,
}

impl<R: Read + Seek> ParsedDeflate<R> {
    pub fn parse(mut src: R) -> io::Result<Self> {
        let mut block_end = Vec::new();
        let mut more = true;
        let mut src_pos = src.stream_position()?;
        let mut out_pos = 0u64;
        while more {
            let mut header = [0u8];
            src.read_exact(&mut header)?;
            let bfinal = header[0] & 1;
            let btype = (header[0] >> 1) & 3;
            match btype {
                // no compression
                0b00 => {
                    let mut len_buf = [0u8; 4];
                    src.read_exact(&mut len_buf)?;
                    more = bfinal == 0;
                    let len = len_buf[0] as u16 | ((len_buf[1] as u16) << 8u16);
                    let nlen = len_buf[2] as u16 | ((len_buf[3] as u16) << 8u16);
                    if len != !nlen {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "invalid nlen {} for len {} at offset {}",
                                nlen, len, src_pos,
                            ),
                        ));
                    }
                    src_pos = src.seek(SeekFrom::Current(len as i64))?;
                    out_pos += len as u64;
                    block_end.push((src_pos, out_pos));
                }
                // compressed
                0b01 | 0b10 => {
                    // rewind to block start
                    src.seek(SeekFrom::Start(src_pos))?;
                    break;
                }
                // reserved
                0b11 => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unexpected reserved BTYPE 11 at offset {}", src_pos),
                    ));
                }
                4u8..=u8::MAX => unreachable!(),
            }
        }
        Ok(Self {
            src,
            block_end,
            more,
        })
    }

    pub fn into_reader(self) -> Result<impl Read + Seek, R> {
        if self.more {
            Err(self.src)
        } else {
            Ok(SeekReadDeflate {
                parsed: self,
                block_idx: 0,
                out_pos: 0,
                seeked: false,
            })
        }
    }
}

struct SeekReadDeflate<R: Read + Seek> {
    parsed: ParsedDeflate<R>,
    block_idx: usize,
    out_pos: u64,
    seeked: bool,
}

impl<R: Read + Seek> SeekReadDeflate<R> {
    pub fn len(&self) -> u64 {
        self.parsed.block_end.last().unwrap().1
    }
}

impl<R: Read + Seek> Read for SeekReadDeflate<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.block_idx == self.parsed.block_end.len() {
                return Ok(0);
            }
            if self.out_pos == self.parsed.block_end[self.block_idx].1 {
                self.block_idx += 1;
                self.seeked = true;
                continue;
            }
            break;
        }
        let input_len = self.parsed.block_end[self.block_idx].1 - self.out_pos;
        if self.seeked {
            self.parsed.src.seek(SeekFrom::Start(
                self.parsed.block_end[self.block_idx].0 - input_len,
            ))?;
            self.seeked = false;
        }
        let len = std::cmp::min(input_len, buf.len() as u64);
        let len = self.parsed.src.read(&mut buf[..len as usize])?;
        self.out_pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for SeekReadDeflate<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.len();
        let seek_error = |msg| Err(io::Error::new(io::ErrorKind::InvalidInput, msg));
        let seek_beyond = |val| {
            seek_error(format!(
                "invalid seek to {} + {}, which is beyond the end of the read-only file",
                len, val,
            ))
        };
        let seek_before = |val| {
            seek_error(format!(
                "invalid seek to -{}, which is before the start of the file",
                val
            ))
        };
        let pos = match pos {
            SeekFrom::Start(pos) => {
                if pos > len {
                    return seek_beyond(pos - len);
                }
                pos
            }
            SeekFrom::Current(delta) => {
                if delta > 0 {
                    let delta = delta as u64;
                    if delta > len - self.out_pos {
                        return seek_beyond(delta - (len - self.out_pos));
                    }
                    self.out_pos + delta
                } else {
                    let delta = (-delta) as u64;
                    if delta > self.out_pos {
                        return seek_before(delta - self.out_pos);
                    }
                    self.out_pos - delta
                }
            }
            SeekFrom::End(val) => {
                if val < 0 {
                    return seek_beyond((-val) as u64);
                }
                let val = val as u64;
                if val > len {
                    return seek_before(val - len);
                }
                len - val
            }
        };
        if pos == self.out_pos {
            return Ok(pos);
        }
        self.seeked = true;
        self.block_idx = self.parsed.block_end.partition_point(|(_, s)| *s <= pos);
        Ok(pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.out_pos)
    }

    fn rewind(&mut self) -> io::Result<()> {
        if self.out_pos != 0 {
            self.out_pos = 0;
            self.block_idx = 0;
            self.seeked = true;
        }
        Ok(())
    }
}

pub fn decode_deflate(src: impl io::BufRead) -> impl Read {
    flate2::bufread::DeflateDecoder::new(src)
}
