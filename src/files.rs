use std::fs;
use std::path::Path;

pub fn exists(file_name:&str) -> Option<&str> {
    if !fs::metadata(file_name).is_ok() {
        return None 
    }
    return Some(file_name)
}

pub fn ensure_directory_exists(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
    }
}

pub fn with_dir(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
    }
}