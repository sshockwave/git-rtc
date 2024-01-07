use gix_object::{decode::LooseHeaderDecodeError, Kind};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum Error {
    Decode(LooseHeaderDecodeError),
    IO(std::io::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

pub struct GitObjectHeader {
    pub kind: Kind,
    pub len: usize,
    pub header_len: usize,
}

const MAX_HEADER_LEN: usize = 10;

pub fn parse_stream_header(reader: &mut impl BufRead) -> Result<(Kind, u64, usize), Error> {
    let mut buffer = Vec::new();
    (&mut *reader)
        .take(MAX_HEADER_LEN as u64)
        .read_until(0, &mut buffer)
        .map_err(Error::IO)?;
    gix_object::decode::loose_header(buffer.as_ref()).map_err(Error::Decode)
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

pub fn parse_seek_header(reader: impl Read + Seek) -> Result<(Kind, u64, impl Read + Seek), Error> {
    let mut reader = BufReader::with_capacity(MAX_HEADER_LEN, reader);
    let (kind, size, consumed_bytes) = parse_stream_header(&mut reader)?;
    let mut reader = OffsetSeek {
        data: reader.into_inner(),
        offset: consumed_bytes,
    };
    reader.seek(SeekFrom::Start(0)).map_err(Error::IO)?;
    Ok((kind, size, reader))
}
