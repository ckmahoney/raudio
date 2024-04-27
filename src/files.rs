use std::fs;
use std::path::Path;

pub fn exists(file_name:&str) -> Option<&str> {
    if !fs::metadata(file_name).is_ok() {
        return None 
    }
    return Some(file_name)
}

pub fn with_dir(path: &str) {
    let path = Path::new(path);

    // Check if the path's final component is likely a file (by checking for an extension)
    let dir = if path.extension().is_some() {
        path.parent().unwrap_or_else(|| Path::new("/"))
    } else {
        path
    };

    if !dir.exists() {
        fs::create_dir_all(dir).expect("Failed to create directory");
    }
}