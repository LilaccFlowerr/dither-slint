use std::collections::HashMap;
use std::path::PathBuf;

pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("dither".to_string(), library_path())])
}
