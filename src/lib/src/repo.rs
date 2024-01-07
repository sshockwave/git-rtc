use gix_hash::ObjectId;
use gix_object::{find::Error as FindError, Kind};
use std::{
    io::{BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
};

// https://git-scm.com/docs/gitrepository-layout
pub struct Repository {
    ref_store: gix_ref::file::Store,
    obj_store: gix_odb::Handle,
    obj_path: PathBuf,
}

fn file_to_stream(reader: impl Read) -> Result<(Kind, u64, impl BufRead), FindError> {
    let mut reader = BufReader::new(git_rtc_fmt::decode_zlib(BufReader::new(reader)));
    let (kind, len, _) = git_rtc_fmt::git::parse_stream_header(&mut reader)?;
    Ok((kind, len, reader))
}

macro_rules! join_path {
    ($path:expr, $first:expr, $($rest:expr),*) => {
        {
            let mut path = $path.join($first);
            $(
                path.push($rest);
            )*
            path
        }
    };
}

impl Repository {
    pub fn at(root: PathBuf) -> std::io::Result<Self> {
        let hash_kind = gix_hash::Kind::Sha1;
        Ok(Self {
            ref_store: gix_ref::file::Store::at(
                root.clone(),
                gix_ref::store::WriteReflog::Normal,
                hash_kind,
            ),
            obj_store: gix_odb::at_opts(
                root.clone(),
                std::iter::empty(),
                gix_odb::store::init::Options {
                    slots: Default::default(),
                    object_hash: hash_kind,
                    use_multi_pack_index: true,
                    current_dir: Some(root.clone()),
                },
            )?,
            obj_path: root.join("objects"),
        })
    }

    pub fn git_dir(&self) -> &Path {
        self.ref_store.git_dir()
    }

    pub fn iter_refs(
        &self,
    ) -> Result<gix_ref::file::iter::Platform, gix_ref::packed::buffer::open::Error> {
        self.ref_store.iter()
    }

    fn loose_object_path(&self, oid: gix_hash::ObjectId) -> PathBuf {
        fn byte_to_hex(b: u8) -> String {
            format!("{:02x}", b)
        }
        fn bytes_to_hex(bytes: &[u8]) -> String {
            bytes.iter().map(|b| byte_to_hex(*b)).collect()
        }
        match oid {
            gix_hash::ObjectId::Sha1(sha1) => join_path!(
                self.obj_path,
                byte_to_hex(sha1[0]),
                bytes_to_hex(&sha1[1..])
            ),
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
    ) -> Result<Option<(Kind, u64, impl BufRead)>, FindError> {
        Ok(match self.open_loose_object(oid)? {
            Some(f) => Some(file_to_stream(f)?),
            None => None,
        })
    }

    pub fn open_loose_object_seek(
        &self,
        oid: ObjectId,
    ) -> Result<Option<(Kind, u64, Result<impl Read + Seek, impl BufRead>)>, FindError> {
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

    pub fn open_packed_object(&self, oid: ObjectId) -> Result<Option<(Kind, Vec<u8>)>, FindError> {
        let mut buffer = Vec::new();
        use gix_object::Find;
        Ok(match self.obj_store.try_find(&oid, &mut buffer)? {
            Some(data) => Some((data.kind, buffer)),
            None => None,
        })
    }
}
