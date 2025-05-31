// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod menu {
    pub mod menubar;
    pub mod settings;
}

mod thread {
    pub mod responses;
    pub mod threads;
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            menu::settings::get_settings,
            menu::settings::save_settings,
            thread::threads::fetch_threads,
            thread::responses::fetch_thread_content,
            thread::responses::fetch_image_as_base64
        ])
        .setup(|app| {
            let app_handle = app.handle(); // AppHandle を取得

            // 非同期タスク用に AppHandle をクローンしてムーブする
            let async_task_app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                println!("[Rust] Attempting to ensure settings are initialized on app setup...");
                match menu::settings::get_settings(async_task_app_handle).await {
                    // クローンしたハンドルを使用
                    Ok(s) => println!("[Rust] Initial settings check OK on setup: {:?}", s),
                    Err(e) => {
                        eprintln!("[Rust] Error during initial settings check on setup: {}", e)
                    }
                }
            });

            // メニュー作成と設定には元の app_handle (またはそのクローン) を使用
            let menu =
                menu::menubar::create_app_menu(&app_handle).expect("Failed to create app menu."); // menuモジュールにあると仮定
            app.set_menu(menu)?; // app を直接使うか、app_handle.set_menu(menu)? でも可

            Ok(())
        })
        .on_menu_event(|app_handle, event| {
            menu::menubar::handle_menu_event(app_handle, event);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
