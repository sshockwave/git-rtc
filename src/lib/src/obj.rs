use crate::hash;

pub enum Kind {
    Blob,
    Tree,
    Commit,
    Tag,
}

pub enum ID {
    SHA1(<hash::sha1::Hasher as hash::Hasher>::Result),
}
