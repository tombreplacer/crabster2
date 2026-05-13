use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::file_info::{get_file_entry, DirListing};

pub struct AppState {
    pub root_dir: PathBuf,
    pub readonly: bool,
    pub hidden: bool,
    pub no_delete: bool,
}

#[derive(Deserialize)]
pub struct PathQuery {
    pub path: Option<String>,
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
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
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

/// Upload files via multipart
pub async fn upload_files(
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
    mut payload: Multipart,
) -> HttpResponse {
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
pub async fn server_info(data: web::Data<AppState>) -> HttpResponse {
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
    data: web::Data<AppState>,
    query: web::Query<PathQuery>,
) -> HttpResponse {
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
    let clean = relative
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();

    let candidate = root.join(&clean);

    // Canonicalize to resolve .. and symlinks
    let canonical_root = root.canonicalize().ok()?;
    let canonical_candidate = if candidate.exists() {
        candidate.canonicalize().ok()?
    } else {
        // For non-existing paths (e.g., upload targets), check parent
        let parent = candidate.parent()?;
        let canonical_parent = parent.canonicalize().ok()?;
        if !canonical_parent.starts_with(&canonical_root) {
            return None;
        }
        let file_name = candidate.file_name()?;
        return Some(canonical_parent.join(file_name));
    };

    if canonical_candidate.starts_with(&canonical_root) {
        Some(canonical_candidate)
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
