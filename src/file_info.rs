use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub mime_type: String,
}

#[derive(Serialize, Debug)]
pub struct DirListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub readonly: bool,
    pub no_delete: bool,
}

pub fn get_file_entry(path: &Path, name: &str) -> Option<FileEntry> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return None,
    };

    let is_symlink = metadata.is_symlink();
    let actual_metadata = if is_symlink {
        fs::metadata(path).unwrap_or(metadata.clone())
    } else {
        metadata.clone()
    };

    let is_dir = actual_metadata.is_dir();
    let size = if is_dir { 0 } else { actual_metadata.len() };

    let modified = actual_metadata
        .modified()
        .ok()
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.to_rfc3339()
        });

    let permissions = get_permissions(&metadata);
    let (owner, group) = get_owner_group(&metadata);

    let mime_type = if is_dir {
        "directory".to_string()
    } else {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    };

    Some(FileEntry {
        name: name.to_string(),
        is_dir,
        is_symlink,
        size,
        modified,
        permissions,
        owner,
        group,
        mime_type,
    })
}

#[cfg(unix)]
fn get_permissions(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let mut perms = String::with_capacity(9);
    let flags = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    for (bit, ch) in &flags {
        perms.push(if mode & bit != 0 { *ch } else { '-' });
    }
    perms
}

#[cfg(windows)]
fn get_permissions(metadata: &fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "readonly".to_string()
    } else {
        "readwrite".to_string()
    }
}

#[cfg(not(any(unix, windows)))]
fn get_permissions(_metadata: &fs::Metadata) -> String {
    "unknown".to_string()
}

#[cfg(unix)]
fn get_owner_group(metadata: &fs::Metadata) -> (String, String) {
    use std::os::unix::fs::MetadataExt;

    let uid = metadata.uid();
    let gid = metadata.gid();

    let owner = unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            uid.to_string()
        } else {
            std::ffi::CStr::from_ptr((*pw).pw_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    let group = unsafe {
        let gr = libc::getgrgid(gid);
        if gr.is_null() {
            gid.to_string()
        } else {
            std::ffi::CStr::from_ptr((*gr).gr_name)
                .to_string_lossy()
                .into_owned()
        }
    };

    (owner, group)
}

#[cfg(not(unix))]
fn get_owner_group(_metadata: &fs::Metadata) -> (String, String) {
    ("N/A".to_string(), "N/A".to_string())
}
