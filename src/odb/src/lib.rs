pub use gix_hash::{Kind as HashType, ObjectId};
pub use gix_object::{find::Error as FindError, Kind as ObjectType};
use std::{
    io::{BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
};

fn file_to_stream(reader: impl Read) -> Result<(ObjectType, u64, impl BufRead), FindError> {
    let mut reader = BufReader::new(git_rtc_fmt::decode_zlib(BufReader::new(reader)));
    let (kind, len, _) = git_rtc_fmt::git::parse_stream_header(&mut reader)?;
    Ok((kind, len, reader))
}

// https://git-scm.com/docs/gitrepository-layout
pub struct ObjectStore {
    obj_store: gix_odb::Handle,
    objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn at(git_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let hash_kind = HashType::Sha1;
        Ok(Self {
            obj_store: gix_odb::at_opts(
                git_dir.as_ref().to_path_buf(),
                std::iter::empty(),
                gix_odb::store::init::Options {
                    slots: Default::default(),
                    object_hash: hash_kind,
                    use_multi_pack_index: true,
                    current_dir: Some(git_dir.as_ref().to_path_buf()),
                },
            )?,
            objects_dir: git_dir.as_ref().join("objects"),
        })
    }

    fn loose_object_path(&self, oid: ObjectId) -> PathBuf {
        fn byte_to_hex(b: u8) -> String {
            format!("{:02x}", b)
        }
        fn bytes_to_hex(bytes: &[u8]) -> String {
            bytes.iter().map(|b| byte_to_hex(*b)).collect()
        }
        match oid {
            ObjectId::Sha1(sha1) => {
                let mut path = self.objects_dir.join(byte_to_hex(sha1[0]));
                path.push(bytes_to_hex(&sha1[1..]));
                path
            }
        }
    }

    fn open_loose_object(&self, oid: ObjectId) -> std::io::Result<Option<std::fs::File>> {
        let loose_path = self.loose_object_path(oid);
        match std::fs::File::open(loose_path) {
            Ok(file) => Ok(Some(file)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(e),
            },
        }
    }

    pub fn open_loose_object_stream(
        &self,
        oid: ObjectId,
    ) -> Result<Option<(ObjectType, u64, impl BufRead)>, FindError> {
        Ok(match self.open_loose_object(oid)? {
            Some(f) => Some(file_to_stream(f)?),
            None => None,
        })
    }

    pub fn open_loose_object_seek(
        &self,
        oid: ObjectId,
    ) -> Result<Option<(ObjectType, u64, Result<impl Read + Seek, impl BufRead>)>, FindError> {
        let mut file = match self.open_loose_object(oid)? {
            Some(f) => f,
            None => return Ok(None),
        };
        git_rtc_fmt::ZlibHeader::parse(&mut file)?;
        let deflate = git_rtc_fmt::ParsedDeflate::parse(file).map_err(|e| Box::new(e))?;
        Ok(Some(match deflate.into_seek_reader() {
            Ok(reader) => {
                let (kind, len, reader) = git_rtc_fmt::git::parse_seek_header(reader)?;
                (kind, len, Ok(reader))
            }
            Err(reader) => {
                let (kind, len, reader) = file_to_stream(reader.into_inner())?;
                (kind, len, Err(reader))
            }
        }))
    }

    pub fn open_packed_object(
        &self,
        oid: ObjectId,
    ) -> Result<Option<(ObjectType, Vec<u8>)>, FindError> {
        let mut buffer = Vec::new();
        use gix_object::Find;
        Ok(match self.obj_store.try_find(&oid, &mut buffer)? {
            Some(data) => Some((data.kind, buffer)),
            None => None,
        })
    }
}
