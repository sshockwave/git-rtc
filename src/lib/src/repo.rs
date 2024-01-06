use crate::obj::{Kind, Store};
use std::{
    io::{Read, Seek},
    path::{Path, PathBuf},
};

pub struct Repository {
    ref_store: gix_ref::file::Store,
    obj_store: gix_odb::Handle,
    obj_path: PathBuf,
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
            gix_hash::ObjectId::Sha1(sha1) => {
                let mut path = self.obj_path.join(byte_to_hex(sha1[0]));
                path.push(bytes_to_hex(&sha1[1..]));
                path
            }
        }
    }

    pub fn open_object(
        &self,
        oid: gix_hash::ObjectId,
    ) -> Result<
        Option<(
            Kind,
            usize,
            Store<impl AsRef<[u8]>, impl Read, impl Read + Seek>,
        )>,
        gix_object::find::Error,
    > {
        let loose_path = self.loose_object_path(oid);
        if loose_path.exists() {
            let mut file = std::fs::File::open(loose_path)?;
            git_rtc_fmt::ZlibHeader::parse(&mut file)?;
            let deflate = git_rtc_fmt::ParsedDeflate::parse(file).map_err(|e| Box::new(e))?;
            let (kind, len, data) = match deflate.to_seek_reader() {
                Ok(seek) => Store::<&[u8], _, _>::Seek { src: seek },
                Err(deflate) => Store::Read {
                    src: git_rtc_fmt::decode_zlib(std::io::BufReader::new(deflate.into_inner())),
                },
            }
            .parse_header()?;
            Ok(Some((
                kind,
                len,
                match data {
                    Store::Buffer { .. } => unreachable!(),
                    Store::Read { src } => Store::Read { src },
                    Store::Seek { src } => Store::Seek { src },
                },
            )))
        } else {
            let mut buffer = Vec::new();
            use gix_object::Find;
            match self.obj_store.try_find(&oid, &mut buffer)? {
                Some(data) => Ok(Some((
                    data.kind,
                    buffer.len(),
                    Store::Buffer { data: buffer },
                ))),
                None => Ok(None),
            }
        }
    }
}
