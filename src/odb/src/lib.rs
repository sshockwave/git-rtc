mod flate;
mod loose;

pub use gix_hash::{Kind as HashType, ObjectId};
pub use gix_object::{find::Error as FindError, Kind as ObjectType};
use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
};

// https://git-scm.com/docs/gitrepository-layout
pub struct ObjectStore {
    obj_store: gix_odb::Handle,
    objects_dir: PathBuf,
    temp_dir: PathBuf,
}

impl ObjectStore {
    pub fn at(git_dir: &Path) -> io::Result<Self> {
        let hash_kind = HashType::Sha1;
        let objects_dir = git_dir.join("objects");
        {
            let mut alternates = objects_dir.clone();
            alternates.push("info");
            alternates.push("alternates");
            if alternates.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    ".git/objects/info/alternates not supported",
                ));
            }
        }
        let temp_dir = objects_dir.join("temp");
        Ok(Self {
            obj_store: gix_odb::at_opts(
                git_dir.to_path_buf(),
                std::iter::empty(),
                gix_odb::store::init::Options {
                    slots: Default::default(),
                    object_hash: hash_kind,
                    use_multi_pack_index: true,
                    current_dir: Some(git_dir.to_path_buf()),
                },
            )?,
            objects_dir,
            temp_dir,
        })
    }

    pub fn open_loose_object_stream(
        &self,
        oid: ObjectId,
    ) -> io::Result<(ObjectType, u64, impl BufRead)> {
        loose::file_to_stream(fs::File::open(loose::object_path(
            self.objects_dir.clone(),
            oid,
        ))?)
    }

    pub fn open_loose_object_seek(
        &self,
        oid: ObjectId,
    ) -> io::Result<(ObjectType, u64, Result<impl Read + Seek, impl BufRead>)> {
        let mut file = fs::File::open(loose::object_path(self.objects_dir.clone(), oid))?;
        flate::ZlibHeader::parse(&mut file)?;
        let deflate = flate::ParsedDeflate::parse(file)?;
        Ok(match deflate.into_reader() {
            Ok(reader) => {
                let (kind, len, reader) = loose::parse_seek_header(reader)?;
                (kind, len, Ok(reader))
            }
            Err(mut reader) => {
                reader.rewind()?;
                let (kind, len, reader) = loose::file_to_stream(reader)?;
                (kind, len, Err(reader))
            }
        })
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

    pub fn write(
        &self,
        object_type: ObjectType,
        hash_type: HashType,
        len: u64,
        compression: bool,
    ) -> io::Result<WriteHandle> {
        use sha1::Digest;
        if !self.temp_dir.exists() {
            std::fs::create_dir_all(&self.temp_dir)?;
        }
        let mut out = WriteHandle {
            out: flate2::write::DeflateEncoder::new(
                tempfile::NamedTempFile::new_in(self.temp_dir.as_path())?,
                if compression {
                    flate2::Compression::best()
                } else {
                    flate2::Compression::none()
                },
            ),
            hash_state: match hash_type {
                HashType::Sha1 => HashState::Sha1(sha1::Sha1::new()),
            },
            objects_dir: self.objects_dir.clone(),
        };
        out.write_all(object_type.as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(len.to_string().as_bytes())?;
        out.write_all(&[0u8])?;
        Ok(out)
    }
}

enum HashState {
    Sha1(sha1::Sha1),
}

pub struct WriteHandle {
    out: flate2::write::DeflateEncoder<tempfile::NamedTempFile>,
    hash_state: HashState,
    objects_dir: PathBuf,
}

impl Write for WriteHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use sha1::Digest;
        match &mut self.hash_state {
            HashState::Sha1(state) => state.update(buf),
        }
        self.out.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

impl WriteHandle {
    pub fn end(self) -> io::Result<ObjectId> {
        use sha1::Digest;
        let oid = match self.hash_state {
            HashState::Sha1(state) => ObjectId::Sha1(state.finalize().into()),
        };
        let file = self.out.finish()?;
        let path = loose::object_path(self.objects_dir, oid);
        if let Err(e) = file.persist_noclobber(&path) {
            match fs::File::open(&path) {
                Ok(old) => {
                    let reader1 = flate::decode_zlib(BufReader::new(old));
                    let mut cur = e.file;
                    cur.rewind()?;
                    let reader2 = flate::decode_zlib(BufReader::new(cur));
                    assert!(streams_equal(reader1, reader2)?, "hash collision detected");
                }
                Err(e2) => {
                    return Err(match e2.kind() {
                        io::ErrorKind::NotFound => e.error,
                        _ => e2,
                    })
                }
            }
        }
        Ok(oid)
    }
}

fn streams_equal(mut reader1: impl Read, mut reader2: impl Read) -> io::Result<bool> {
    let mut buffer1 = [0; 4096];
    let mut buffer2 = [0; 4096];
    let mut begin1 = 0usize;
    let mut end1 = 0usize;
    let mut begin2 = 0usize;
    let mut end2 = 0usize;

    loop {
        if begin1 == end1 {
            begin1 = 0;
            end1 = reader1.read(&mut buffer1)?;
        }
        if begin2 == end2 {
            begin2 = 0;
            end2 = reader2.read(&mut buffer2)?;
        }

        let cmp_len = std::cmp::min(end1 - begin1, end2 - begin2);
        if cmp_len == 0 {
            return Ok(begin1 == end1 && begin2 == end2);
        }
        let new_begin1 = begin1 + cmp_len;
        let new_begin2 = begin2 + cmp_len;
        if buffer1[begin1..new_begin1] != buffer2[begin2..new_begin2] {
            return Ok(false);
        }
        begin1 = new_begin1;
        begin2 = new_begin2;
    }
}
