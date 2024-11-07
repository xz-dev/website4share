pub mod utils;
use futures::TryStreamExt;

use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use md5;
use serde_json::Map;
use std::str;
use tokio::fs;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

#[get("/list")]
async fn list() -> Result<impl Responder> {
    // list all dir in tmp_dir
    let dir = utils::get_tmp_dir().await;
    let children = utils::get_folder_list(&dir).await?;
    // read childern as str list
    let json_list: Vec<String> = children
        .iter()
        .map(|file| {
            file.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    Ok(web::Json(json_list))
}

#[post("/new/{name}")]
async fn new(name: web::Path<String>) -> Result<impl Responder> {
    // create new dir in tmp_dir
    let dir = utils::get_tmp_dir().await;
    let new_dir = dir.join(&name.to_string());
    fs::create_dir_all(&new_dir).await?;
    Ok(HttpResponse::Ok())
}

#[delete("/delete/{pname}")]
async fn delete(pname: web::Path<String>) -> Result<impl Responder> {
    // delete dir in tmp_dir
    let dir = utils::get_tmp_dir().await;
    let delete_dir = dir.join(pname.to_string());
    fs::remove_dir_all(&delete_dir).await?;
    Ok(HttpResponse::Ok())
}

// list pasteboard file contents with unix timestamp
// like [{content: "hello", timestamp: 123456}]
#[get("/list_pasteboard/{pname}")]
async fn list_pasteboard(pname: web::Path<String>) -> Result<impl Responder> {
    let dir = utils::get_pasteboard_dir(&pname).await?;
    let pasteboard_files = utils::get_file_list(&dir).await?;
    let mut json_list: Vec<Map<String, serde_json::Value>> = Vec::new();

    for file in pasteboard_files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        let content = fs::read_to_string(&file).await.unwrap_or_default();
        let timestamp = file
            .metadata()
            .and_then(|m| m.modified()) // or m.created() if you want the creation time
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0)
            .to_string();
        let mut map = Map::new();
        map.insert("id".to_string(), serde_json::Value::String(name));
        map.insert("content".to_string(), serde_json::Value::String(content));
        map.insert(
            "timestamp".to_string(),
            serde_json::Value::String(timestamp),
        );
        json_list.push(map);
    }

    Ok(web::Json(json_list))
}

#[post("/new_pasteboard/{pname}")]
async fn new_pasteboard(pname: web::Path<String>, payload: web::Payload) -> Result<impl Responder> {
    // create new pasteboard file in tmp_dir
    let dir = utils::get_pasteboard_dir(&pname).await?;
    // unix timestamp + md5 hash
    let content = str::from_utf8(&payload.to_bytes().await?.to_vec())?.to_string();
    let name = format!(
        "{:?}_{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        md5::compute(&content)
    );
    let pasteboard_file = dir.join(name);
    fs::write(&pasteboard_file, &content).await?;
    Ok(HttpResponse::Ok())
}

#[delete("/delete_pasteboard/{pname}/{id}")]
async fn delete_pasteboard(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, id) = path.into_inner();
    // delete pasteboard file in tmp_dir
    let dir = utils::get_pasteboard_dir(&pname).await?;
    let delete_file = dir.join(id);
    fs::remove_file(&delete_file).await?;
    Ok(HttpResponse::Ok())
}

#[get("/list_files/{pname}")]
async fn list_files(pname: web::Path<String>) -> Result<impl Responder> {
    // list all files file in tmp_dir
    let dir = utils::get_files_dir(&pname).await?;
    let files_files = utils::get_file_list(&dir).await?;
    let mut json_list: Vec<Map<String, serde_json::Value>> = Vec::new();

    for file in files_files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        let timestamp = file
            .metadata()
            .and_then(|m| m.modified()) // or m.created() if you want the creation time
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0)
            .to_string();
        let mut map = Map::new();
        map.insert("name".to_string(), serde_json::Value::String(name));
        map.insert(
            "timestamp".to_string(),
            serde_json::Value::String(timestamp),
        );
        json_list.push(map);
    }

    Ok(web::Json(json_list))
}

#[post("/new_file/{pname}/{name}/{offset}")]
async fn new_file(
    path: web::Path<(String, String, String)>,
    mut payload: Multipart,
) -> Result<impl Responder> {
    let (pname, name, offset) = path.into_inner();
    let dir = utils::get_files_dir(&pname).await?;
    let file_path = dir.join(&name);
    let offset = offset.parse::<u64>().unwrap_or_default();

    // Use OpenOptions to open the file so that data can be appended when the file exists
    let mut file = if offset == 0 {
        fs::File::create(&file_path).await?
    } else {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)
            .await?
    };
    if offset == 0 {
        // touch file_name.uploading file for marking the file is uploading
        let uploading_file = file_path.with_extension("uploading");
        fs::write(&uploading_file, "").await?;
    }

    // seek to the offset
    file.seek(std::io::SeekFrom::Start(offset)).await?;

    // Write data to the file
    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file.write_all(&data).await?;
        }
    }

    Ok(HttpResponse::Ok())
}

// return the file size, to let client know if the file is uploaded, and where to resume
#[get("/check_new_file/{pname}/{name}")]
async fn check_new_file(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, name) = path.into_inner();
    let dir = utils::get_files_dir(&pname).await?;
    let file_path = dir.join(&name);

    let size = match fs::metadata(&file_path).await {
        Ok(metadata) => {
            let uploading_file = file_path.with_extension("uploading");
            if uploading_file.exists() {
                metadata.len()
            } else {
                0
            }
        }
        Err(_) => 0,
    };

    let json = serde_json::json!({
        "size": size,
    });

    Ok(web::Json(json))
}

#[post("/done_new_file/{pname}/{name}")]
async fn done_new_file(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, name) = path.into_inner();
    let dir = utils::get_files_dir(&pname).await?;
    let file_path = dir.join(&name);
    let uploading_file = file_path.with_extension("uploading");
    fs::remove_file(&uploading_file).await?;
    Ok(HttpResponse::Ok())
}

#[delete("/delete_files/{pname}/{name}")]
async fn delete_files(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, name) = path.into_inner();
    // delete files file in tmp_dir
    let dir = utils::get_files_dir(&pname).await?;
    let delete_file = dir.join(name.to_string());
    std::fs::remove_file(&delete_file)?;
    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let data_dir = utils::_get_tmp_dir();
        App::new()
            .service(list)
            .service(new)
            .service(delete)
            .service(list_pasteboard)
            .service(new_pasteboard)
            .service(delete_pasteboard)
            .service(list_files)
            .service(new_file)
            .service(check_new_file)
            .service(done_new_file)
            .service(delete_files)
            .service(
                Files::new("/files", &data_dir)
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(Files::new("/", "./static").index_file("index.html"))
            .service(Files::new("/", "./static").show_files_listing())
    })
    .bind(utils::get_listen_addr())?
    .run()
    .await
}
