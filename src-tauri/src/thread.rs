use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Wry};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadItem {
    id: String,
    title: String,
    response_count: u32,
    created_at: String, // 表示用に文字列として定義
}

// レスポンスアイテムの構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseItem {
    id: String,
    author: String,
    content: String,
    created_at: String,
}

pub fn get_data_file_path(filename: &str) -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "ユーザーの設定ディレクトリが見つかりませんでした。".to_string())?;
    let app_config_subdir = config_dir.join("tulip-browser"); // 設定ファイルと同じディレクトリを使用
    if !app_config_subdir.exists() {
        std::fs::create_dir_all(&app_config_subdir).map_err(|e| {
            format!(
                "データ用ディレクトリ '{}' の作成に失敗しました: {}",
                app_config_subdir.display(),
                e
            )
        })?;
    }
    Ok(app_config_subdir.join(filename))
}

// 新しいTauriコマンド: スレッド一覧をファイルから取得
#[tauri::command]
pub async fn fetch_threads(app_handle: AppHandle<Wry>) -> Result<Vec<ThreadItem>, String> {
    let data_file_name = "threads_mock.json";
    let data_file_path = get_data_file_path(data_file_name)?;

    println!(
        "[Rust fetch_threads] スレッドデータをファイルから取得します: {}",
        data_file_path.display()
    );

    // もしファイルが存在しなければ、サンプルデータで作成
    if !data_file_path.exists() {
        println!(
            "[Rust fetch_threads] {} が存在しないため、サンプルデータで作成します。",
            data_file_path.display()
        );
        let sample_threads_json = r#"[
          {
            "id": "sample001",
            "title": "サンプルスレッド１ (初回起動時作成)",
            "response_count": 10,
            "created_at": "2025/05/30 11:00"
          },
          {
            "id": "sample002",
            "title": "Tauri 機能テスト中",
            "response_count": 5,
            "created_at": "2025/05/30 11:05"
          }
        ]"#;
        fs::write(&data_file_path, sample_threads_json).map_err(|e| {
            format!(
                "サンプルスレッドファイル '{}' の書き込みに失敗しました: {}",
                data_file_path.display(),
                e
            )
        })?;
        println!("[Rust fetch_threads] サンプルスレッドファイルを作成しました。");
    }

    // ファイルを読み込み
    let data = fs::read_to_string(&data_file_path).map_err(|e| {
        format!(
            "スレッドファイル '{}' の読み込みに失敗しました: {}",
            data_file_path.display(),
            e
        )
    })?;

    // JSONをパース
    let threads: Vec<ThreadItem> = serde_json::from_str(&data).map_err(|e| {
        format!(
            "スレッドデータのJSONパースに失敗しました (ファイル: {}): {}",
            data_file_path.display(),
            e
        )
    })?;

    println!(
        "[Rust fetch_threads] {} 個のスレッドを読み込みました。",
        threads.len()
    );
    Ok(threads)
}
