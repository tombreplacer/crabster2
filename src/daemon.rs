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
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let path = Path::new(&home).join(".crabster").join("instances");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path
}

pub fn generate_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Use last 6 chars of hex nanoseconds, reversed for more randomness
    format!("{:x}", now).chars().rev().take(6).collect()
}

pub fn save_instance(id: &str, port: u16, bind: &str, dir: PathBuf) {
    let instance = Instance {
        id: id.to_string(),
        pid: process::id(),
        port,
        bind: bind.to_string(),
        dir,
        started: Utc::now(),
    };

    let path = get_state_dir().join(format!("{}.json", id));
    if let Ok(json) = serde_json::to_string(&instance) {
        fs::write(path, json).ok();
    }
}

fn get_log_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let path = Path::new(&home).join(".crabster").join("logs");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path
}

pub fn get_log_path(id: &str) -> PathBuf {
    get_log_dir().join(format!("{}.log", id))
}

use std::io;
use std::io::Read;

#[cfg(unix)]
pub fn daemonize() -> io::Result<(String, bool, Option<i32>)> {
    use libc::{fork, setsid, umask, chdir, pipe};

    let id = generate_id();
    let mut fds = [0i32; 2];
    
    unsafe {
        if pipe(fds.as_mut_ptr()) < 0 {
            return Err(io::Error::last_os_error());
        }

        let pid = fork();
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        
        if pid > 0 {
            // Parent process
            libc::close(fds[1]); // Close write end
            let mut buf = [0u8; 1024];
            let n = libc::read(fds[0], buf.as_mut_ptr() as *mut libc::c_void, buf.len());
            libc::close(fds[0]);
            
            if n > 0 {
                let msg = String::from_utf8_lossy(&buf[..n as usize]);
                if msg == "OK" {
                    return Ok((id, true, None));
                } else {
                    return Err(io::Error::new(io::ErrorKind::Other, msg));
                }
            } else if n == 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "Daemon child exited unexpectedly"));
            } else {
                return Err(io::Error::last_os_error());
            }
        }

        // Child process
        libc::close(fds[0]); // Close read end
        
        if setsid() < 0 {
            process::exit(1);
        }

        umask(0);
        
        let root = std::ffi::CString::new("/").unwrap();
        chdir(root.as_ptr());

        // We'll redirect stdio later, after notifying parent or failing
        // For now, return the write end of the pipe
        Ok((id, false, Some(fds[1])))
    }
}

#[cfg(unix)]
pub fn notify_success(pipe_fd: i32, id: &str) {
    unsafe {
        libc::write(pipe_fd, "OK".as_ptr() as *const libc::c_void, 2);
        libc::close(pipe_fd);
        
        // Redirect stdio to log file
        let log_path = get_log_path(id);
        let log_path_c = std::ffi::CString::new(log_path.to_string_lossy().as_bytes()).unwrap();
        let fd = libc::open(log_path_c.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND, 0o644);
        if fd >= 0 {
            libc::dup2(fd, 1); // stdout
            libc::dup2(fd, 2); // stderr
            if fd > 2 {
                libc::close(fd);
            }
        }
        
        // stdin still to /dev/null
        let dev_null = std::ffi::CString::new("/dev/null").unwrap();
        let stdin_fd = libc::open(dev_null.as_ptr(), libc::O_RDONLY);
        if stdin_fd >= 0 {
            libc::dup2(stdin_fd, 0);
            if stdin_fd > 2 {
                libc::close(stdin_fd);
            }
        }
    }
}

#[cfg(not(unix))]
pub fn notify_success(_pipe_fd: i32, _id: &str) {}

#[cfg(unix)]
pub fn notify_error(pipe_fd: i32, err: &str) {
    unsafe {
        libc::write(pipe_fd, err.as_ptr() as *const libc::c_void, err.len());
        libc::close(pipe_fd);
        process::exit(1);
    }
}

#[cfg(not(unix))]
pub fn notify_error(_pipe_fd: i32, _err: &str) {
    process::exit(1);
}

#[cfg(not(unix))]
pub fn daemonize() -> std::io::Result<(String, bool, Option<i32>)> {
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
    {
        let _ = pid;
        false
    }
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
            #[cfg(not(unix))]
            {
                let _ = instance;
                return Err("Stopping instances is only supported on Unix systems".to_string());
            }
            fs::remove_file(path).ok();
            return Ok(());
        }
    }
    Err("Failed to read instance info".to_string())
}

pub fn tail_logs(id: &str, follow: bool) -> io::Result<()> {
    let log_path = get_log_path(id);
    if !log_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Log file not found for instance '{}'", id)));
    }

    if !follow {
        let content = fs::read_to_string(log_path)?;
        println!("{}", content);
        return Ok(());
    }

    // Follow mode (simple implementation)
    let mut file = fs::File::open(&log_path)?;
    let mut buffer = String::new();
    
    // Start from the end of the file
    use std::io::Seek;
    file.seek(io::SeekFrom::End(0))?;

    loop {
        let mut reader = io::BufReader::new(&file);
        let n = reader.read_to_string(&mut buffer)?;
        if n > 0 {
            print!("{}", buffer);
            buffer.clear();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
