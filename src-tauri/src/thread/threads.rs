use chrono::DateTime;
use chrono_tz::Asia::Tokyo;
use html_escape::decode_html_entities;
use serde::{Deserialize, Deserializer, Serialize};

// APIから直接受け取るJSONの各要素に対応する構造体
#[derive(Deserialize, Debug, Clone)]
struct ApiThreadItem {
    // "thread" フィールドが数値または文字列の場合に対応するため、カスタムデシリアライザを使用
    #[serde(deserialize_with = "deserialize_thread_id_to_string")]
    thread: String, // スレッドIDを文字列として統一
    title: String,
    number: u32, // レス数
    date: i64,   // Unixタイムスタンプ (最終更新日時など)
}

// フロントエンドに渡すためのスレッド情報の構造体 (既存のものを確認・使用)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadItem {
    id: String,          // スレッドID
    title: String,       // タイトル
    response_count: u32, // レス数
    created_at: String,  // フォーマットされた日時文字列 (ここではAPIの 'date' を使用)
    date: i64,
}

// "thread" フィールドのカスタムデシリアライザ
// JSON内で数値でも文字列でも送られてくる可能性がある "thread" IDを常にStringとして読み込む
fn deserialize_thread_id_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)] // 型が一致するものを試す
    enum ThreadIdValue {
        Str(String),
        Num(serde_json::Number), // 数値をまずは serde_json::Number で受ける
    }

    match ThreadIdValue::deserialize(deserializer)? {
        ThreadIdValue::Str(s) => Ok(s),
        ThreadIdValue::Num(n) => Ok(n.to_string()), // 数値を文字列に変換
    }
}

// Unixタイムスタンプ (i64) を "YYYY/MM/DD HH:MM" 形式のJST日時文字列に変換するヘルパー関数
fn format_timestamp_from_i64(timestamp_secs: i64) -> String {
    // DateTime::from_timestamp は DateTime<Utc> を返す
    if let Some(datetime_utc) = DateTime::from_timestamp(timestamp_secs, 0) {
        // UTCのDateTimeをJST (Asia/Tokyo) のDateTimeに変換
        let datetime_jst = datetime_utc.with_timezone(&Tokyo);
        // JSTのDateTimeを指定されたフォーマットの文字列に変換
        return datetime_jst.format("%Y/%m/%d %H:%M").to_string();
    }
    "日付不明".to_string()
}

#[tauri::command]
pub async fn fetch_threads() -> Result<Vec<ThreadItem>, String> {
    let json_url = "https://tulipplantation.com/tulipplantation/subject.json";
    println!(
        "[Rust fetch_threads] スレッド一覧を取得します: {}",
        json_url
    );

    let client = reqwest::Client::new();
    let api_items_result = match client.get(json_url).send().await {
        // ... (HTTPリクエストとエラーハンドリング部分は変更なし) ...
        Ok(response) => {
            if response.status().is_success() {
                response.json::<Vec<ApiThreadItem>>().await
            } else {
                let err_msg = format!("HTTPエラー: {} (URL: {})", response.status(), json_url);
                eprintln!("[Rust fetch_threads] {}", err_msg);
                return Err(err_msg);
            }
        }
        Err(e) => {
            let err_msg = format!("リクエストに失敗しました (URL: {}): {}", json_url, e);
            eprintln!("[Rust fetch_threads] {}", err_msg);
            return Err(err_msg);
        }
    };

    let api_items = match api_items_result {
        Ok(items) => items,
        Err(e) => {
            let err_msg = format!(
                "JSONのパースまたはリクエスト内容のエラー (URL: {}): {}",
                json_url, e
            );
            eprintln!("[Rust fetch_threads] {}", err_msg);
            return Err(err_msg);
        }
    };

    // ApiThreadItem からフロントエンド用の ThreadItem に変換する際にタイトルをデコード
    let threads: Vec<ThreadItem> = api_items
        .into_iter()
        .map(|api_item| ThreadItem {
            id: api_item.thread,
            // ★★★ HTMLエンティティをデコード ★★★
            title: decode_html_entities(&api_item.title).into_owned(),
            response_count: api_item.number,
            created_at: format_timestamp_from_i64(api_item.date),
            date: api_item.date,
        })
        .collect();

    println!(
        "[Rust fetch_threads] {} 個のスレッドを取得・変換しました。",
        threads.len()
    );
    Ok(threads)
}
