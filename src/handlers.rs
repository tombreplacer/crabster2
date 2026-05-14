use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::file_info::{get_file_entry, DirListing};

pub struct AppState {
    pub root_dir: PathBuf,
    pub readonly: bool,
    pub hidden: bool,
    pub no_delete: bool,
    pub auth: Option<String>,
}

#[derive(Deserialize)]
pub struct PathQuery {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

pub fn check_auth(req: &HttpRequest, data: &web::Data<AppState>) -> Result<(), HttpResponse> {
    if let Some(expected) = &data.auth {
        if let Some(cookie) = req.cookie("crabster_auth") {
            if cookie.value() == expected {
                return Ok(());
            }
        }
        return Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required",
            "needsAuth": true
        })));
    }
    Ok(())
}

pub async fn auth_login(
    data: web::Data<AppState>,
    body: web::Json<AuthRequest>,
) -> HttpResponse {
    if let Some(expected) = &data.auth {
        if body.code == *expected {
            let cookie = actix_web::cookie::Cookie::build("crabster_auth", expected.clone())
                .path("/")
                .http_only(true)
                .finish();
            return HttpResponse::Ok()
                .cookie(cookie)
                .json(serde_json::json!({"success": true}));
        }
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid code"
        }));
    }
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

/// Serve the embedded HTML page
pub async fn index_handler() -> HttpResponse {
    let html = include_str!("../static/index.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// List files in directory
pub async fn list_files(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    let rel_path = query.path.as_deref().unwrap_or("");
    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let full_path = resolve_safe_path(&data.root_dir, &rel_path);
    let full_path = match full_path {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if !full_path.is_dir() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Directory not found"
        }));
    }

    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(&full_path) {
        Ok(rd) => rd,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Cannot read directory: {}", e)
            }));
        }
    };

    for entry in read_dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files unless --hidden flag
        if !data.hidden && name.starts_with('.') {
            continue;
        }

        if let Some(file_entry) = get_file_entry(&entry.path(), &name) {
            entries.push(file_entry);
        }
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let listing = DirListing {
        path: rel_path,
        entries,
        readonly: data.readonly,
        no_delete: data.no_delete,
    };

    HttpResponse::Ok().json(listing)
}

/// Download a file
pub async fn download_file(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    let rel_path = req.match_info().query("path");
    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let full_path = match resolve_safe_path(&data.root_dir, &rel_path) {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if !full_path.is_file() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        }));
    }

    let filename = full_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    let encoded_filename = percent_encoding::percent_encode(
        filename.as_bytes(),
        percent_encoding::NON_ALPHANUMERIC,
    ).to_string();

    match tokio::fs::read(&full_path).await {
        Ok(content) => HttpResponse::Ok()
            .content_type(mime)
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"; filename*=UTF-8''{}", filename, encoded_filename),
            ))
            .body(content),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Cannot read file: {}", e)
        })),
    }
}

/// Preview a file inline (with Range support for video streaming)
pub async fn preview_file(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    let rel_path = req.match_info().query("path");
    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let full_path = match resolve_safe_path(&data.root_dir, &rel_path) {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if !full_path.is_file() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        }));
    }

    let file_size = match tokio::fs::metadata(&full_path).await {
        Ok(m) => m.len(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Cannot stat file: {}", e)
            }));
        }
    };

    let mut mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    if mime == "application/octet-stream" {
        if file_size == 0 {
            mime = "text/plain".to_string();
        } else if let Ok(mut f) = tokio::fs::File::open(&full_path).await {
            let mut buf = [0; 512];
            if let Ok(n) = f.read(&mut buf).await {
                if n > 0 && !buf[..n].contains(&0) {
                    mime = "text/plain".to_string();
                }
            }
        }
    }

    let filename = full_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let encoded_filename = percent_encoding::percent_encode(
        filename.as_bytes(),
        percent_encoding::NON_ALPHANUMERIC,
    ).to_string();

    let content_disposition = format!(
        "inline; filename=\"{}\"; filename*=UTF-8''{}",
        filename, encoded_filename
    );

    // Check for Range header
    if let Some(range_header) = req.headers().get("Range") {
        let range_str = match range_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest().body("Invalid Range header");
            }
        };

        // Parse "bytes=start-end"
        if let Some(range) = parse_range(range_str, file_size) {
            let (start, end) = range;
            let chunk_size = end - start + 1;

            let mut file = match tokio::fs::File::open(&full_path).await {
                Ok(f) => f,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Cannot open file: {}", e)
                    }));
                }
            };

            if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Seek error: {}", e)
                }));
            }

            let mut buf = vec![0u8; chunk_size as usize];
            if let Err(e) = file.read_exact(&mut buf).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Read error: {}", e)
                }));
            }

            return HttpResponse::PartialContent()
                .content_type(mime)
                .insert_header(("Content-Disposition", content_disposition))
                .insert_header(("Accept-Ranges", "bytes"))
                .insert_header((
                    "Content-Range",
                    format!("bytes {}-{}/{}", start, end, file_size),
                ))
                .insert_header(("Content-Length", chunk_size.to_string()))
                .body(buf);
        } else {
            // Invalid range
            return HttpResponse::RangeNotSatisfiable()
                .insert_header(("Content-Range", format!("bytes */{}", file_size)))
                .finish();
        }
    }

    // No Range header — serve full file
    match tokio::fs::read(&full_path).await {
        Ok(content) => HttpResponse::Ok()
            .content_type(mime)
            .insert_header(("Content-Disposition", content_disposition))
            .insert_header(("Accept-Ranges", "bytes"))
            .insert_header(("Content-Length", file_size.to_string()))
            .body(content),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Cannot read file: {}", e)
        })),
    }
}

/// Upload files via multipart
pub async fn upload_files(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
    mut payload: Multipart,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    if data.readonly {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Server is in read-only mode"
        }));
    }

    let rel_path = query.path.as_deref().unwrap_or("");
    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let target_dir = match resolve_safe_path(&data.root_dir, &rel_path) {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if !target_dir.is_dir() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Target path is not a directory"
        }));
    }

    let mut uploaded: Vec<String> = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd.clone(),
            None => continue,
        };
        let filename = match content_disposition.get_filename() {
            Some(f) => sanitize_filename(f),
            None => continue,
        };

        if filename.is_empty() {
            continue;
        }

        let filepath = target_dir.join(&filename);

        if filepath.exists() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": format!("File '{}' already exists", filename)
            }));
        }

        let mut file = match tokio::fs::File::create(&filepath).await {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Cannot create file '{}': {}", filename, e)
                }));
            }
        };

        while let Some(Ok(chunk)) = field.next().await {
            if let Err(e) = file.write_all(&chunk).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Write error for '{}': {}", filename, e)
                }));
            }
        }

        uploaded.push(filename);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "uploaded": uploaded,
        "count": uploaded.len()
    }))
}

/// Delete a file or empty directory
pub async fn delete_file(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    if data.readonly || data.no_delete {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Deletion is disabled"
        }));
    }

    let rel_path = req.match_info().query("path");
    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let full_path = match resolve_safe_path(&data.root_dir, &rel_path) {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if !full_path.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        }));
    }

    let result = if full_path.is_dir() {
        tokio::fs::remove_dir(&full_path).await
    } else {
        tokio::fs::remove_file(&full_path).await
    };

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "deleted": rel_path
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Cannot delete: {}", e)
        })),
    }
}

/// Server info endpoint
pub async fn server_info(
    req: HttpRequest,
    data: web::Data<AppState>
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    let root = data
        .root_dir
        .canonicalize()
        .unwrap_or_else(|_| data.root_dir.clone());

    HttpResponse::Ok().json(serde_json::json!({
        "name": "crabster",
        "version": env!("CARGO_PKG_VERSION"),
        "root": root.to_string_lossy(),
        "readonly": data.readonly,
        "no_delete": data.no_delete,
        "hidden": data.hidden,
    }))
}

/// Create a new directory
pub async fn create_dir(
    req: HttpRequest,
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
    if let Err(e) = check_auth(&req, &data) { return e; }
    if data.readonly {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Server is in read-only mode"
        }));
    }

    let rel_path = query.path.as_deref().unwrap_or("");
    if rel_path.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Path is required"
        }));
    }

    let rel_path = percent_decode_str(rel_path)
        .decode_utf8_lossy()
        .to_string();

    let full_path = match resolve_safe_path(&data.root_dir, &rel_path) {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid path"
            }));
        }
    };

    if full_path.exists() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Path already exists"
        }));
    }

    match tokio::fs::create_dir_all(&full_path).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "created": rel_path
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Cannot create directory: {}", e)
        })),
    }
}

/// Resolve a relative path safely within root_dir (prevent path traversal)
fn resolve_safe_path(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(root);
    
    // Iterate through components and manually resolve them to prevent escaping the root
    for component in Path::new(relative).components() {
        match component {
            std::path::Component::Normal(c) => path.push(c),
            std::path::Component::ParentDir => {
                // Try to pop. If we're already at root, this is an illegal traversal attempt
                if path == root {
                    return None;
                }
                path.pop();
            }
            std::path::Component::RootDir => {
                // Ignore absolute paths in the relative part, treat as relative to root
            }
            std::path::Component::CurDir => {
                // Ignore '.'
            }
            _ => {}
        }
    }

    // Double check that the final path still starts with our root
    if path.starts_with(root) {
        Some(path)
    } else {
        None
    }
}

/// Sanitize a filename to prevent directory traversal
fn sanitize_filename(name: &str) -> String {
    let name = name.replace('\\', "/");
    let name = name.rsplit('/').next().unwrap_or(&name);
    let name = name.trim_start_matches('.');
    name.to_string()
}

/// Parse HTTP Range header value like "bytes=0-1023" into (start, end)
fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start: u64;
    let end: u64;

    if parts[0].is_empty() {
        // suffix range: bytes=-500
        let suffix: u64 = parts[1].parse().ok()?;
        if suffix > file_size {
            return None;
        }
        start = file_size - suffix;
        end = file_size - 1;
    } else {
        start = parts[0].parse().ok()?;
        if parts[1].is_empty() {
            // open-ended: bytes=500-
            end = file_size - 1;
        } else {
            end = parts[1].parse().ok()?;
        }
    }

    if start > end || start >= file_size {
        return None;
    }

    let end = end.min(file_size - 1);
    Some((start, end))
}
