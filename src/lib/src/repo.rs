use git_rtc_fmt::git::{parse_seek_header, parse_stream_header};
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

fn file_to_stream(reader: impl Read + Seek) -> Result<(Kind, u64, impl BufRead), FindError> {
    let mut reader = BufReader::new(git_rtc_fmt::decode_zlib(BufReader::new(reader)));
    let (kind, size, _) = parse_stream_header(&mut reader)?;
    Ok((kind, size, reader))
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

    fn open_object_from_packs(
        &self,
        oid: gix_hash::ObjectId,
    ) -> Result<Option<(Kind, Vec<u8>)>, gix_object::find::Error> {
        let mut buffer = Vec::new();
        use gix_object::Find;
        Ok(
            if let Some(data) = self.obj_store.try_find(&oid, &mut buffer)? {
                Some((data.kind, buffer))
            } else {
                None
            },
        )
    }

    pub fn open_object_seek(
        &self,
        oid: gix_hash::ObjectId,
    ) -> Result<
        Option<(
            Kind,
            u64,
            SeekResult<impl AsRef<[u8]>, impl BufRead, impl Read + Seek>,
        )>,
        gix_object::find::Error,
    > {
        let loose_path = self.loose_object_path(oid);
        Ok(if loose_path.exists() {
            let mut file = std::fs::File::open(loose_path)?;
            git_rtc_fmt::ZlibHeader::parse(&mut file)?;
            let deflate = git_rtc_fmt::ParsedDeflate::parse(file).map_err(|e| Box::new(e))?;
            Some(match deflate.into_seek_reader() {
                Ok(reader) => {
                    let (kind, len, reader) = parse_seek_header(reader)?;
                    (kind, len, SeekResult::Seek(reader))
                }
                Err(reader) => {
                    let (kind, len, reader) = file_to_stream(reader.into_inner())?;
                    (kind, len, SeekResult::Stream(reader))
                }
            })
        } else {
            self.open_object_from_packs(oid)?
                .map(|(kind, data)| (kind, data.len() as u64, SeekResult::Buffer(data)))
        })
    }
}

enum SeekResult<B, R, S> {
    Buffer(B),
    Stream(R),
    Seek(S),
}
