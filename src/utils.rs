use std::{env, path::PathBuf};
use tokio::fs;

// get listen address or env LISTEN_ADDR
pub fn get_listen_addr() -> String {
    env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:8080".to_string())
}

// get tmp dir or env TMPDIR
pub fn _get_tmp_dir() -> PathBuf {
    env::var("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir().join("website4share"))
}
pub async fn get_tmp_dir() -> PathBuf {
    let dir = _get_tmp_dir();
    fs::create_dir_all(&dir).await.unwrap();
    dir
}

pub async fn get_pasteboard_dir(pname: &str) -> Result<PathBuf, std::io::Error> {
    let dir = get_tmp_dir().await.join(pname).join("pasteboard");
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}

pub async fn get_files_dir(pname: &str) -> Result<PathBuf, std::io::Error> {
    let dir = get_tmp_dir().await.join(pname).join("files");
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}

// get folder list in dir
pub async fn get_folder_list(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut folder_list = Vec::new();
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            folder_list.push(path);
        }
    }
    Ok(folder_list)
}

// get file list in dir
pub async fn get_file_list(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut file_list = Vec::new();
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            file_list.push(path);
        }
    }
    Ok(file_list)
}
