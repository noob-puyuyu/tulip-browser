// src/setting.rs
use dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Wry;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreBuilder; // Manager と Runtime を削除

// AppSettings 構造体 (変更なし)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    theme: String,
    font_size: u32,
}
// AppSettings のデフォルト値 (変更なし)
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            font_size: 14,
        }
    }
}

const SETTINGS_STORE_PATH_FILENAME: &str = "setting.json";
const SETTINGS_KEY: &str = "app_settings";

// 設定ファイルのフルパスを取得し、ディレクトリがなければ作成する関数 (ログ強化版のまま)
fn get_store_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "ユーザーの設定ディレクトリが見つかりませんでした。".to_string())?;
    let app_config_subdir = config_dir.join("tulip-browser");
    if !app_config_subdir.exists() {
        std::fs::create_dir_all(&app_config_subdir).map_err(|e| {
            format!(
                "ディレクトリ '{}' の作成に失敗しました: {}",
                app_config_subdir.display(),
                e
            )
        })?;
    }
    Ok(app_config_subdir.join(SETTINGS_STORE_PATH_FILENAME))
}

fn create_and_save_default_settings(
    store: &Arc<tauri_plugin_store::Store<Wry>>,
    file_path: &PathBuf,
) -> Result<AppSettings, String> {
    let default_settings = AppSettings::default();
    println!(
        "[Rust create_and_save_default_settings] 使用するデフォルト設定: {:?}",
        default_settings
    );
    let value_to_set = serde_json::to_value(&default_settings).map_err(|e_json| {
        format!(
            "デフォルト設定のJSONへのシリアライズに失敗しました: {}",
            e_json
        )
    })?;
    println!("[Rust create_and_save_default_settings] メモリ上のストアにデフォルト値をセットします (キー: '{}')...", SETTINGS_KEY);
    store.set(SETTINGS_KEY.to_string(), value_to_set);
    println!(
        "[Rust create_and_save_default_settings] ストアをファイル '{}' に保存します...",
        file_path.display()
    );
    if let Err(save_err) = store.save() {
        return Err(format!(
            "デフォルト設定のストア '{}' への保存に失敗しました: {}",
            file_path.display(),
            save_err
        ));
    }
    println!("[Rust create_and_save_default_settings] デフォルト設定をファイルに保存しました。デフォルト設定を返します。");
    Ok(default_settings)
}

#[tauri::command]
pub async fn get_settings(app_handle: AppHandle<Wry>) -> Result<AppSettings, String> {
    println!("[Rust get_settings] コマンドが呼び出されました。");
    let path = get_store_path()?;
    println!("[Rust get_settings] ストアパス: '{}'", path.display());

    let store = StoreBuilder::new(&app_handle, path.clone())
        .build()
        .map_err(|e| format!("ストアの構築に失敗しました: {}", e))?;
    println!("[Rust get_settings] ストア構築成功。");

    println!(
        "[Rust get_settings] ストア '{}' の再読み込みを試みます...",
        path.display()
    );
    match store.reload() {
        Ok(_) => {
            println!("[Rust get_settings] ストア '{}' の再読み込み成功（またはファイルが存在せず何もしなかったか、中身が空だった）。", path.display());
            if let Some(value) = store.get(SETTINGS_KEY) {
                println!(
                    "[Rust get_settings] キー '{}' から設定値を読み込みました。",
                    SETTINGS_KEY
                );
                return serde_json::from_value(value.clone()).map_err(|e_json| {
                    format!(
                        "キー '{}' からの設定のデシリアライズに失敗しました: {}",
                        SETTINGS_KEY, e_json
                    )
                });
            } else {
                println!("[Rust get_settings] ストアにキー '{}' が見つかりません (reload成功後)。デフォルト設定を作成して保存します。", SETTINGS_KEY);
                return create_and_save_default_settings(&store, &path);
            }
        }
        Err(e) => {
            // store.reload() がエラーを返した場合の処理
            let error_string = e.to_string();
            // ★★★★★ 以下のデバッグ出力でエラー e の実際の型や内容を確認してください ★★★★★
            eprintln!(
                "[Rust get_settings] store.reload() がエラーを返しました (詳細): {:?}",
                e
            );
            eprintln!(
                "[Rust get_settings] store.reload() がエラーを返しました (メッセージ): {}",
                error_string
            );

            // 暫定対応: エラーメッセージの文字列で「ファイルが見つからない」状況を判断
            // "os error 2" は "No such file or directory" を意味します。
            // より安全なのは、上記 {:?} の出力で実際のErrorバリアントを確認し、それでmatchすることです。
            if error_string.contains("os error 2")
                || error_string
                    .to_lowercase()
                    .contains("no such file or directory")
                || error_string
                    .to_lowercase()
                    .contains("such file or directory")
            {
                // より一般的なフレーズ
                println!("[Rust get_settings] ストアファイル '{}' が見つかりません (reloadエラーの文字列検査より判断)。デフォルト設定を作成して保存します。", path.display());
                return create_and_save_default_settings(&store, &path);
            } else {
                // その他の予期せぬ reload エラー
                let err_msg = format!(
                    "ストア '{}' の再読み込み中に予期せぬエラーが発生しました (上記詳細参照): {}",
                    path.display(),
                    error_string
                );
                eprintln!("[Rust get_settings] {}", err_msg);
                return Err(err_msg);
            }
        }
    }
}

// save_settings コマンド (前回のログ強化版のまま)
#[tauri::command]
pub async fn save_settings(
    app_handle: AppHandle<Wry>,
    settings: AppSettings,
) -> Result<(), String> {
    println!(
        "[Rust save_settings] コマンドが呼び出されました。設定: {:?}",
        settings
    );
    let path = get_store_path()?;
    println!("[Rust save_settings] ストアパス: '{}'", path.display());

    let store = StoreBuilder::new(&app_handle, path.clone())
        .build()
        .map_err(|e| format!("ストアの構築に失敗しました: {}", e))?;
    println!("[Rust save_settings] ストア構築成功。");

    let value_to_set = serde_json::to_value(&settings).map_err(|e_json| e_json.to_string())?;
    println!("[Rust save_settings] メモリ上のストアに値をセットします...");
    store.set(SETTINGS_KEY.to_string(), value_to_set);

    println!(
        "[Rust save_settings] ストアをファイル '{}' に保存します...",
        path.display()
    );
    match store.save() {
        Ok(_) => {
            println!(
                "[Rust save_settings] 設定をストア '{}' に保存しました。",
                path.display()
            );
            if let Err(e_emit) = app_handle.emit_to("main", "settings_changed", &settings) {
                eprintln!(
                    "[Rust save_settings] settings_changed イベントの送信に失敗しました: {}",
                    e_emit
                );
            }
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("ストア '{}' の保存に失敗しました: {}", path.display(), e);
            eprintln!("[Rust save_settings] {}", err_msg);
            Err(err_msg)
        }
    }
}
