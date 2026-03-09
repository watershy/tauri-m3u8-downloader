use std::path::{Path, PathBuf};

pub fn join<P1: AsRef<Path>, P2: AsRef<Path>>(base: P1, append: P2) -> PathBuf {
    base.as_ref().join(append)
}