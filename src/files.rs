use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn exists(file_name: &str) -> Option<&str> {
  if !fs::metadata(file_name).is_ok() {
    return None;
  }
  return Some(file_name);
}

pub fn with_dir(path: &str) {
  let path = Path::new(path);

  // Determine if running in GitHub Actions
  let adjusted_path = if env::var("GITHUB_ACTIONS").is_ok() {
    // Use the GitHub Actions workspace path if available
    let github_workspace = env::var("GITHUB_WORKSPACE").expect("GITHUB_WORKSPACE not set");
    PathBuf::from(github_workspace).join(path)
  } else {
    // Use the path as-is if not in GitHub Actions
    path.to_path_buf()
  };

  // Check if the final component is likely a file (by checking for an extension)
  let dir = if adjusted_path.extension().is_some() {
    adjusted_path.parent().unwrap_or_else(|| Path::new("/"))
  } else {
    adjusted_path.as_path()
  };

  // Create the directory if it doesn't exist
  if !dir.exists() {
    fs::create_dir_all(dir).expect("Failed to create directory");
  }
}
