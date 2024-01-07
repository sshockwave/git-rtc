use super::{ObjectId, ObjectType};
use std::{
    io::{self, BufRead, BufReader, Read, Seek},
    path::PathBuf,
};

pub fn object_path(mut objects_dir: PathBuf, oid: ObjectId) -> PathBuf {
    fn byte_to_hex(b: u8) -> String {
        format!("{:02x}", b)
    }
    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| byte_to_hex(*b)).collect()
    }
    match oid {
        ObjectId::Sha1(sha1) => {
            objects_dir.push(byte_to_hex(sha1[0]));
            objects_dir.push(bytes_to_hex(&sha1[1..]));
            objects_dir
        }
    }
}

pub fn file_to_stream(reader: impl Read) -> io::Result<(ObjectType, u64, impl BufRead)> {
    let mut reader = BufReader::new(crate::flate::decode_zlib(BufReader::new(reader)));
    let (kind, len, _) = parse_stream_header(&mut reader)?;
    Ok((kind, len, reader))
}

const MAX_HEADER_LEN: usize = 10;

pub fn parse_stream_header(reader: &mut impl BufRead) -> io::Result<(ObjectType, u64, usize)> {
    let mut buffer = Vec::new();
    (&mut *reader)
        .take(MAX_HEADER_LEN as u64)
        .read_until(0, &mut buffer)?;
    gix_object::decode::loose_header(buffer.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

struct OffsetSeek<T: Read + Seek> {
    data: T,
    offset: usize,
}

impl<T: Read + Seek> Read for OffsetSeek<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

impl<T: Read + Seek> Seek for OffsetSeek<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        use io::SeekFrom::*;
        match pos {
            Start(pos) => self.data.seek(Start(pos + self.offset as u64)),
            End(pos) => self.data.seek(End(pos)),
            Current(pos) => self.data.seek(Current(pos)),
        }
    }
}

pub fn parse_seek_header(
    reader: impl Read + Seek,
) -> io::Result<(ObjectType, u64, impl Read + Seek)> {
    let mut reader = BufReader::with_capacity(MAX_HEADER_LEN, reader);
    let (kind, size, consumed_bytes) = parse_stream_header(&mut reader)?;
    let mut reader = OffsetSeek {
        data: reader.into_inner(),
        offset: consumed_bytes,
    };
    reader.seek(io::SeekFrom::Start(0))?;
    Ok((kind, size, reader))
}
