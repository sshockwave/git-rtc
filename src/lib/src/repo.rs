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
    hash_kind: gix_hash::Kind,
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
            hash_kind: gix_hash::Kind::Sha1,
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
}
