#[allow(unused_imports)]
use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{AppHandle, Manager, Runtime}; // AppHandle, Runtime を追加

use tauri::WebviewWindowBuilder; // WebviewWindowBuilder を追加

pub fn create_app_menu<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // 「設定...」メニューアイテムを作成
    let settings_item = MenuItemBuilder::new("設定...")
        .id("settings") // IDはイベント処理で使います
        // .accelerator("CmdOrCtrl+,") // 必要であればアクセラレータを設定
        .build(app_handle)?;

    // メインメニューの作成 (トップレベルに「設定...」アイテムのみ)
    // macOSの場合、これはアプリケーションメニュー（アプリ名）の下に配置されることが期待されます。
    // Windows/Linuxでは、トップレベルにこのアイテムのみが表示されます。
    let menu = Menu::with_items(app_handle, &[&settings_item])?;

    Ok(menu)
}

// メニューイベントを処理する関数 (Tauri v2) - 「設定」イベントのみ処理

#[allow(unused)]
pub fn handle_menu_event<R: Runtime>(app_handle: &AppHandle<R>, event: MenuEvent) {
    match event.id.as_ref() {
        "settings" => {
            println!("Menu: 'settings' clicked.");
            let settings_window_label = "settings_window";

            if let Some(existing_window) = app_handle.get_webview_window(settings_window_label) {
                if let Err(e) = existing_window.set_focus() {
                    eprintln!("Failed to focus settings window: {}", e);
                }
                // macOSの場合の追加対応 (前回同様)
                #[cfg(target_os = "macos")]
                if let Err(e) = app_handle
                    .plugin_global::<tauri::plugin::Window>()
                    .activate_window(settings_window_label)
                {
                    eprintln!("Failed to activate settings window on macOS: {:?}", e);
                }
            } else {
                match WebviewWindowBuilder::new(
                    app_handle,
                    settings_window_label,
                    tauri::WebviewUrl::App("settings.html".into()),
                )
                .title("設定")
                .inner_size(1700.0, 900.0)
                .resizable(true) // ★★★ この行を false から true に変更 ★★★
                // .min_inner_size(300.0, 200.0) // 必要であれば最小サイズも指定できます
                .build()
                {
                    Ok(window) => {
                        println!("Settings window created.");
                        #[cfg(target_os = "macos")]
                        if let Err(e) = window.set_focus() {
                            eprintln!(
                                "Failed to focus newly created settings window on macOS: {}",
                                e
                            );
                        }
                    }
                    Err(e) => eprintln!("Failed to create settings window: {}", e),
                }
            }
        }
        // ... (他のメニューアイテムの処理) ...
        other_id => {
            // ...
        }
    }
}
