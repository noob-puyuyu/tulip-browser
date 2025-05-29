use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// ダミーのスレッド一覧を返すコマンド
#[tauri::command]
pub fn get_dummy_threads() -> Vec<ThreadItem> {
    println!("[Rust] Returning dummy threads."); // ★追加：返すデータ数を確認
    vec![
        ThreadItem {
            id: "dummy001".to_string(),
            title: "【Tauri】Tauriを使ったアプリ開発について語るスレ".to_string(),
            response_count: 123,
            created_at: "2025/05/29 10:00".to_string(),
        },
        ThreadItem {
            id: "dummy002".to_string(),
            title: "Rustの非同期処理完全に理解した".to_string(),
            response_count: 45,
            created_at: "2025/05/28 14:20".to_string(),
        },
        ThreadItem {
            id: "dummy003".to_string(),
            title: "おすすめのキーボード教えて".to_string(),
            response_count: 567,
            created_at: "2025/05/27 08:00".to_string(),
        },
        ThreadItem {
            id: "dummy004".to_string(),
            title: "今日のランチ何食べた？".to_string(),
            response_count: 8,
            created_at: "2025/05/29 12:35".to_string(),
        },
    ]
}

#[tauri::command]
pub fn get_dummy_responses(thread_id: String) -> Vec<ResponseItem> {
    println!(
        "[Rust] get_dummy_responses called for thread_id: {}",
        thread_id
    );
    let mut responses_map: HashMap<String, Vec<ResponseItem>> = HashMap::new();

    responses_map.insert(
        "dummy001".to_string(),
        vec![
            ResponseItem {
                id: "res001_01".to_string(),
                author: "名無しさん１".to_string(),
                content: "Tauri、いいですよね!Rustでフロントエンドも書ける感覚が好きです。"
                    .to_string(),
                created_at: "2025/05/29 10:05".to_string(),
            },
            ResponseItem {
                id: "res001_02".to_string(),
                author: "名無しさん２".to_string(),
                content: "わかる。ビルドサイズも小さいし、パフォーマンスも良い気がする。"
                    .to_string(),
                created_at: "2025/05/29 10:15".to_string(),
            },
            ResponseItem {
                id: "res001_03".to_string(),
                author: "名無しさん３".to_string(),
                content: "まだ始めたばかりだけど、ドキュメントも充実してるね。".to_string(),
                created_at: "2025/05/29 10:30".to_string(),
            },
        ],
    );

    responses_map.insert(
        "dummy002".to_string(),
        vec![
            ResponseItem {
                id: "res002_01".to_string(),
                author: "プログラマーA".to_string(),
                content: "完全に理解した（翌日には忘れてる）".to_string(),
                created_at: "2025/05/28 14:22".to_string(),
            },
            ResponseItem {
                id: "res002_02".to_string(),
                author: "プログラマーB".to_string(),
                content: "tokioとかasync-stdとか、エコシステムも豊富で楽しい。".to_string(),
                created_at: "2025/05/28 14:30".to_string(),
            },
        ],
    );

    responses_map.insert(
        "dummy003".to_string(),
        vec![ResponseItem {
            id: "res003_01".to_string(),
            author: "キーボードマニア".to_string(),
            content: "HHKB一択でしょう!".to_string(),
            created_at: "2025/05/27 08:05".to_string(),
        }],
    );
    // "dummy004" にはレスポンスがないケースも用意
    // responses_map.insert("dummy004".to_string(), vec![]);

    match responses_map.get(&thread_id) {
        Some(responses) => responses.clone(),
        _ => {
            // 対応するスレッドIDのレスポンスがない場合は空のVecを返すか、エラーメッセージ用の特別なレスポンスを返す
            println!("[Rust] No responses found for thread_id: {}", thread_id);
            vec![ResponseItem {
                id: "error_res".to_string(),
                author: "システム".to_string(),
                content: format!(
                    "スレッドID '{}' のレスポンスは見つかりませんでした。",
                    thread_id
                ),
                created_at: "".to_string(),
            }]
        }
    }
}
