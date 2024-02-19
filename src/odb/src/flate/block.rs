use ::std::io;

pub struct Index {
    blocks: Vec<(bool, u64, u64)>, // (compressed, src pos, out pos) of the end of the block
}

fn read_bytes<const N: usize>(mut file: impl io::Read) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn invalid_data<T, E>(error: E) -> io::Result<T>
where
    E: Into<Box<dyn ::std::error::Error + Send + Sync>>,
{
    Err(io::Error::new(io::ErrorKind::InvalidData, error))
}

impl Index {
    fn from_file(mut file: impl io::BufRead + io::Seek) -> io::Result<Self> {
        let mut blocks = Vec::new();
        let mut more = true;
        let mut src_pos = file.stream_position()?;
        let mut out_pos = 0u64;
        while more {
            let (bfinal, btype) = {
                let mut header = read_bytes::<1>(file)?;
                (header[0] & 1, (header[0] >> 1) & 3)
            };
            more = bfinal == 0;
            let compressed = match btype {
                // no compression
                0b00 => {
                    let len = u16::from_le_bytes(read_bytes(file)?);
                    let nlen = u16::from_le_bytes(read_bytes(file)?);
                    if len != !nlen {
                        return invalid_data(format!(
                            "invalid nlen {} for len {} at block with offset {}",
                            nlen, len, src_pos,
                        ));
                    }
                    src_pos = file.seek(io::SeekFrom::Current(len as i64))?;
                    out_pos += len as u64;
                    false
                }
                // compressed with fixed Huffman codes
                0b01 => {
                    file.seek(io::SeekFrom::Start(src_pos))?;
                    todo!();
                    true
                }
                // compressed with dynamic Huffman codes
                0b10 => {
                    file.seek(io::SeekFrom::Start(src_pos))?;
                    todo!();
                    true
                }
                // reserved (error)
                0b11 => {
                    return invalid_data(format!(
                        "unexpected reserved BTYPE 11 at block with offset {}",
                        src_pos
                    ));
                }
                4u8.. => unreachable!(),
            };
            if compressed {
                src_pos = file.stream_position()?;
            }
            blocks.push((compressed, src_pos, out_pos));
        }
        Ok(Self { blocks })
    }
}
