use std::io::{BufRead, BufReader, Cursor, Error, Read, Result, Seek, SeekFrom};

pub use gix_object::Kind;

struct OffsetBuffer<T: AsRef<[u8]>> {
    data: T,
    offset: usize,
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for OffsetBuffer<T> {
    fn as_ref(&self) -> &[u8] {
        &self.data.as_ref()[self.offset..]
    }
}

struct OffsetSeek<T: Read + Seek> {
    data: T,
    offset: usize,
}

impl<T: Read + Seek> Read for OffsetSeek<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.read(buf)
    }
}

impl<T: Read + Seek> Seek for OffsetSeek<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(pos) => self.data.seek(SeekFrom::Start(pos + self.offset as u64)),
            SeekFrom::End(pos) => self.data.seek(SeekFrom::End(pos)),
            SeekFrom::Current(pos) => self.data.seek(SeekFrom::Current(pos)),
        }
    }
}

fn loose_header_from_bufread(reader: &mut impl BufRead) -> Result<(Kind, usize, usize)> {
    let mut buffer = Vec::new();
    let bytes1 = reader.by_ref().take(10).read_until(b' ', &mut buffer)?;
    buffer.pop(); // remove trailing space
    let kind = Kind::from_bytes(buffer.as_ref()).map_err(Error::other)?;

    let mut buffer = Vec::new();
    let bytes2 = reader.by_ref().take(64).read_until(0, &mut buffer)?;
    buffer.pop(); // remove trailing null
    let size = btoi::btoi(buffer.as_ref()).map_err(Error::other)?;
    Ok((kind, size, bytes1 + bytes2))
}

pub enum Store<V, R, S> {
    Buffer { data: V },
    Read { src: R },
    Seek { src: S },
}

impl<V: AsRef<[u8]>, R: Read, S: Read + Seek> Store<V, R, S> {
    pub fn parse_header(
        self,
    ) -> Result<(
        Kind,
        usize,
        Store<impl AsRef<[u8]>, impl Read, impl Read + Seek>,
    )> {
        match self {
            Self::Buffer { data } => {
                let (kind, size, consumed_bytes) =
                    gix_object::decode::loose_header(data.as_ref()).map_err(Error::other)?;
                let data = OffsetBuffer {
                    data,
                    offset: consumed_bytes,
                };
                Ok((kind, size as usize, Store::Buffer { data }))
            }
            Self::Read { src } => {
                let mut reader = BufReader::with_capacity(20, src);
                let (kind, size, _consumed_bytes) = loose_header_from_bufread(&mut reader)?;
                let buf: Vec<_> = reader.buffer().into();
                Ok((
                    kind,
                    size,
                    Store::Read {
                        src: Cursor::new(buf).chain(reader.into_inner()),
                    },
                ))
            }
            Self::Seek { src } => {
                let mut reader = BufReader::with_capacity(20, src);
                let (kind, size, consumed_bytes) = loose_header_from_bufread(&mut reader)?;
                let mut src = OffsetSeek {
                    data: reader.into_inner(),
                    offset: consumed_bytes,
                };
                src.seek(SeekFrom::Start(0))?;
                Ok((kind, size, Store::Seek { src }))
            }
        }
    }
}
