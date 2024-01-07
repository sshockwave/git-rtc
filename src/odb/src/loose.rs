use super::{ObjectId, ObjectType};
use std::{
    io,
    path::{Path, PathBuf},
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

pub fn file_to_stream(
    reader: impl io::Read,
) -> Result<(ObjectType, u64, impl io::BufRead), crate::FindError> {
    let mut reader = io::BufReader::new(crate::flate::decode_zlib(io::BufReader::new(reader)));
    let (kind, len, _) = git_rtc_fmt::git::parse_stream_header(&mut reader)?;
    Ok((kind, len, reader))
}
