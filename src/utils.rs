use std::{env, fs, path::PathBuf};

// get listen address or env LISTEN_ADDR
pub fn get_listen_addr() -> String {
    env::var("LISTEN_ADDR").unwrap_or("127.0.0.1:8080".to_string())
}

// get tmp dir or env TMPDIR
pub fn get_tmp_dir() -> PathBuf {
    let dir = env::var("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir().join("website4share"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn get_pasteboard_dir(pname: &str) -> Result<PathBuf, std::io::Error> {
    let dir = get_tmp_dir().join(pname).join("pasteboard");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_files_dir(pname: &str) -> Result<PathBuf, std::io::Error> {
    let dir = get_tmp_dir().join(pname).join("files");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// get folder list in dir
pub fn get_folder_list(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut folder_list = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            folder_list.push(path);
        }
    }
    Ok(folder_list)
}

// get file list in dir
pub fn get_file_list(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut file_list = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            file_list.push(path);
        }
    }
    Ok(file_list)
}
