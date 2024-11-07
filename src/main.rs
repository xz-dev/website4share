pub mod utils;
use futures::TryStreamExt;

use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use md5;
use serde::Deserialize;
use serde_json::Map;
use std::{io::Write, str};

#[get("/list")]
async fn list() -> Result<impl Responder> {
    // list all dir in tmp_dir
    let dir = utils::get_tmp_dir();
    let children = utils::get_folder_list(&dir)?;
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

#[derive(Deserialize)]
struct NewDir {
    name: String,
}

#[post("/new")]
async fn new(payload: web::Json<NewDir>) -> Result<impl Responder> {
    // create new dir in tmp_dir
    let dir = utils::get_tmp_dir();
    let new_dir = dir.join(&payload.name);
    std::fs::create_dir_all(&new_dir)?;
    Ok(HttpResponse::Ok())
}

#[delete("/delete/{pname}")]
async fn delete(pname: web::Path<String>) -> Result<impl Responder> {
    // delete dir in tmp_dir
    let dir = utils::get_tmp_dir();
    let delete_dir = dir.join(pname.to_string());
    std::fs::remove_dir_all(&delete_dir)?;
    Ok(HttpResponse::Ok())
}

// list pasteboard file contents with unix timestamp
// like [{content: "hello", timestamp: 123456}]
#[get("/list_pasteboard/{pname}")]
async fn list_pasteboard(pname: web::Path<String>) -> Result<impl Responder> {
    let dir = utils::get_pasteboard_dir(&pname)?;
    let pasteboard_files = utils::get_file_list(&dir)?;
    let json_list: Vec<Map<String, serde_json::Value>> = pasteboard_files
        .iter()
        .map(|file| {
            let name = file
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string();
            let content = std::fs::read_to_string(file).unwrap_or_default();
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
            map
        })
        .collect();
    Ok(web::Json(json_list))
}

#[post("/new_pasteboard/{pname}")]
async fn new_pasteboard(pname: web::Path<String>, payload: web::Payload) -> Result<impl Responder> {
    // create new pasteboard file in tmp_dir
    let dir = utils::get_pasteboard_dir(&pname)?;
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
    std::fs::write(&pasteboard_file, &content)?;
    Ok(HttpResponse::Ok())
}

#[delete("/delete_pasteboard/{pname}/{id}")]
async fn delete_pasteboard(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, id) = path.into_inner();
    // delete pasteboard file in tmp_dir
    let dir = utils::get_pasteboard_dir(&pname)?;
    let delete_file = dir.join(id);
    std::fs::remove_file(&delete_file)?;
    Ok(HttpResponse::Ok())
}

// list files file name with unix timestamp
#[get("/list_files/{pname}")]
async fn list_files(pname: web::Path<String>) -> Result<impl Responder> {
    // list all files file in tmp_dir
    let dir = utils::get_files_dir(&pname)?;
    let files_files = utils::get_file_list(&dir)?;
    // read files_files as str list
    let json_list: Vec<Map<String, serde_json::Value>> = files_files
        .iter()
        .map(|file| {
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
            map
        })
        .collect();
    Ok(web::Json(json_list))
}

#[post("/new_file/{pname}/{name}")]
async fn new_file(
    path: web::Path<(String, String)>,
    mut payload: Multipart,
) -> Result<impl Responder> {
    let (pname, name) = path.into_inner();
    // create new files file in tmp_dir
    let dir = utils::get_files_dir(&pname)?;
    let file_path = dir.join(&name);

    while let Ok(Some(mut field)) = payload.try_next().await {
        // File::create is blocking operation, use threadpool
        let file_path_clone = file_path.clone();
        let mut f = web::block(move || std::fs::File::create(file_path_clone)).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || {
                f.and_then(|mut f| {
                    f.write_all(&data)?;
                    Ok(f)
                })
            })
            .await?;
        }
    }

    Ok(HttpResponse::Ok())
}

#[delete("/delete_files/{pname}/{name}")]
async fn delete_files(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (pname, name) = path.into_inner();
    // delete files file in tmp_dir
    let dir = utils::get_files_dir(&pname)?;
    let delete_file = dir.join(name.to_string());
    std::fs::remove_file(&delete_file)?;
    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let data_dir = utils::get_tmp_dir();
        App::new()
            .service(list)
            .service(new)
            .service(delete)
            .service(list_pasteboard)
            .service(new_pasteboard)
            .service(delete_pasteboard)
            .service(list_files)
            .service(new_file)
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
