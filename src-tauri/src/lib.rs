use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::Manager;

// 启动后端服务
fn start_backend_server(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        // 获取资源目录
        let resource_dir = app_handle
            .path_resolver()
            .resource_dir()
            .expect("failed to resolve resource dir");

        let backend_path = resource_dir.join("gitmentor-backend.exe");

        println!("Starting backend server: {:?}", backend_path);

        if !backend_path.exists() {
            println!("Backend executable not found: {:?}", backend_path);
            return;
        }

        match Command::new(&backend_path)
            .current_dir(&resource_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                println!("Backend server started successfully");

                // 等待进程结束
                match child.wait() {
                    Ok(status) => println!("Backend exited with status: {}", status),
                    Err(e) => println!("Error waiting for backend: {}", e),
                }
            }
            Err(e) => {
                println!("Failed to start backend server: {}", e);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 启动后端服务
            let app_handle = app.handle();
            start_backend_server(app_handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
