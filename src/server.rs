use actix_web::{web, App, HttpServer};

use crate::handlers::{
    self, AppState,
};

pub async fn start_server(
    bind: &str,
    port: u16,
    root_dir: std::path::PathBuf,
    readonly: bool,
    hidden: bool,
    no_delete: bool,
    auth: Option<String>,
    daemon_id: Option<String>,
    notify_fd: Option<i32>,
) -> std::io::Result<()> {
    let root_dir = root_dir.canonicalize().unwrap_or(root_dir);

    let state = web::Data::new(AppState {
        root_dir: root_dir.clone(),
        readonly,
        hidden,
        no_delete,
        auth: auth.clone(),
    });

    if daemon_id.is_none() {
        println!();
        println!("  🦀 \x1b[1;38;5;208mCrabster\x1b[0m v{}", env!("CARGO_PKG_VERSION"));
        println!("  ─────────────────────────────────");
        println!("  📂 Serving:   \x1b[36m{}\x1b[0m", root_dir.display());
        println!("  🌐 Listening: \x1b[32mhttp://{}:{}\x1b[0m", bind, port);
        if readonly {
            println!("  🔒 Mode:      \x1b[33mread-only\x1b[0m");
        } else if no_delete {
            println!("  🛡️  Mode:      \x1b[33mno-delete\x1b[0m");
        } else {
            println!("  ✏️  Mode:      \x1b[32mread-write\x1b[0m");
        }
        if auth.is_some() {
            println!("  🔐 Auth:      \x1b[32menabled\x1b[0m");
        }
        println!("  ─────────────────────────────────");
        println!("  Press \x1b[1mCtrl+C\x1b[0m to stop");
        println!();
    }

    let server_result = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::new("%r -> %s (%T) \"%{User-Agent}i\""))
            .app_data(state.clone())
            .app_data(web::PayloadConfig::new(512 * 1024 * 1024)) // 512MB max
            .route("/", web::get().to(handlers::index_handler))
            .route("/api/files", web::get().to(handlers::list_files))
            .route("/api/download/{path:.*}", web::get().to(handlers::download_file))
            .route("/api/preview/{path:.*}", web::get().to(handlers::preview_file))
            .route("/api/upload", web::post().to(handlers::upload_files))
            .route("/api/delete/{path:.*}", web::delete().to(handlers::delete_file))
            .route("/api/info", web::get().to(handlers::server_info))
            .route("/api/mkdir", web::post().to(handlers::create_dir))
            .route("/api/auth", web::post().to(handlers::auth_login))
    })
    .bind(format!("{}:{}", bind, port));

    match server_result {
        Ok(server) => {
            if let Some(id) = &daemon_id {
                crate::daemon::save_instance(id, port, bind, root_dir);
            }
            if let Some(fd) = notify_fd {
                crate::daemon::notify_success(fd, &daemon_id.unwrap());
            }
            server.run().await
        }
        Err(e) => {
            if let Some(fd) = notify_fd {
                crate::daemon::notify_error(fd, &e.to_string());
            }
            Err(e)
        }
    }
}
