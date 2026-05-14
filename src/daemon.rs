use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub pid: u32,
    pub port: u16,
    pub bind: String,
    pub dir: PathBuf,
    pub started: DateTime<Utc>,
}

fn get_state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = Path::new(&home).join(".crabster").join("instances");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path
}

fn generate_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Use last 6 chars of hex nanoseconds, reversed for more randomness
    format!("{:x}", now).chars().rev().take(6).collect()
}

use std::io;

#[cfg(unix)]
pub fn daemonize(port: u16, bind: &str, dir: PathBuf) -> io::Result<String> {
    use libc::{fork, setsid, umask, chdir, close, open, O_RDWR};

    let id = generate_id();
    
    unsafe {
        let pid = fork();
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        if pid > 0 {
            // Parent process
            return Ok(id);
        }

        // Child process
        if setsid() < 0 {
            process::exit(1);
        }

        umask(0);
        
        let root = std::ffi::CString::new("/").unwrap();
        chdir(root.as_ptr());

        // Redirect stdio to /dev/null or log files
        let dev_null = std::ffi::CString::new("/dev/null").unwrap();
        let fd = open(dev_null.as_ptr(), O_RDWR);
        if fd >= 0 {
            libc::dup2(fd, 0);
            libc::dup2(fd, 1);
            libc::dup2(fd, 2);
            if fd > 2 {
                close(fd);
            }
        }
    }

    // Save instance info
    let instance = Instance {
        id: id.clone(),
        pid: process::id(),
        port,
        bind: bind.to_string(),
        dir,
        started: Utc::now(),
    };

    let path = get_state_dir().join(format!("{}.json", id));
    let json = serde_json::to_string(&instance).unwrap();
    fs::write(path, json).ok();

    // The child continues and returns "" to signify it's the daemon
    Ok("".to_string())
}

#[cfg(not(unix))]
pub fn daemonize(_port: u16, _bind: &str, _dir: PathBuf) -> std::io::Result<String> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Daemon mode is only supported on Unix systems"))
}

pub fn list_instances() -> Vec<Instance> {
    let dir = get_state_dir();
    let mut instances = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(instance) = serde_json::from_str::<Instance>(&content) {
                    // Check if process is still running
                    if is_pid_alive(instance.pid) {
                        instances.push(instance);
                    } else {
                        // Cleanup stale file
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }
    }
    instances.sort_by_key(|i| i.started);
    instances
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
    #[cfg(not(unix))]
    false
}

pub fn stop_instance(id: &str) -> Result<(), String> {
    let path = get_state_dir().join(format!("{}.json", id));
    if !path.exists() {
        return Err(format!("Instance '{}' not found", id));
    }

    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(instance) = serde_json::from_str::<Instance>(&content) {
            #[cfg(unix)]
            unsafe {
                if libc::kill(instance.pid as i32, libc::SIGTERM) != 0 {
                    return Err(format!("Failed to stop process {}", instance.pid));
                }
            }
            fs::remove_file(path).ok();
            return Ok(());
        }
    }
    Err("Failed to read instance info".to_string())
}
