use chrono::{DateTime, NaiveDateTime, Utc};
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

// フロントエンドに渡すためのスレッド情報の構造体 (既存のものを確認・使用)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreadItem {
    id: String,          // スレッドID
    title: String,       // タイトル
    response_count: u32, // レス数
    created_at: String,  // フォーマットされた日時文字列 (ここではAPIの 'date' を使用)
}

// Unixタイムスタンプ (i64) を "YYYY/MM/DD HH:MM" 形式の文字列に変換するヘルパー関数
fn format_timestamp_from_i64(timestamp_secs: i64) -> String {
    if let Some(naive_dt) = NaiveDateTime::from_timestamp_opt(timestamp_secs, 0) {
        let datetime_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
        // 必要であればここで日本時間 (JST) に変換
        // use chrono_tz::Asia::Tokyo;
        // let datetime_jst = datetime_utc.with_timezone(&Tokyo);
        // return datetime_jst.format("%Y/%m/%d %H:%M").to_string();
        return datetime_utc.format("%Y/%m/%d %H:%M").to_string(); // UTCのまま表示
    }
    "日付不明".to_string()
}

// レスポンスアイテムの構造体
#[derive(Debug, Serialize, Clone)] // フロントエンドに渡すので Serialize は必須
pub struct ResponseItem {
    id: String,           // レス番号 (例: "1", "2")
    author: String,       // 名前欄
    mail: String,         // メール欄
    created_at: String,   // 日付とIDとBE等を含む文字列全体、またはパースした日付部分
    user_id_info: String, // IDやBEなどの部分
    content: String,      // 本文 (HTMLが含まれる)
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
        })
        .collect();

    println!(
        "[Rust fetch_threads] {} 個のスレッドを取得・変換しました。",
        threads.len()
    );
    Ok(threads)
}

#[tauri::command]
pub async fn fetch_thread_content(thread_id: String) -> Result<Vec<ResponseItem>, String> {
    if thread_id.is_empty() {
        return Err("スレッドIDが指定されていません。".to_string());
    }

    let dir_prefix = if thread_id.len() >= 4 {
        &thread_id[0..4]
    } else {
        return Err(format!(
            "スレッドID '{}' が短すぎるため、ディレクトリを特定できません。",
            thread_id
        ));
    };

    let dat_file_url = format!(
        "https://tulipplantation.com/tulipplantation/thread/{}/{}.dat",
        dir_prefix, thread_id
    );

    println!(
        "[Rust fetch_thread_content] スレッド内容を取得します (ID: {}): {}",
        thread_id, dat_file_url
    );

    let client = reqwest::Client::new();
    // HTTPリクエストを行い、レスポンスを取得
    let response = match client.get(&dat_file_url).send().await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("リクエスト失敗 (URL: {}): {}", dat_file_url, e)),
    };

    // HTTPステータスコードを確認
    if !response.status().is_success() {
        return Err(format!(
            "HTTPエラー {} (URL: {})",
            response.status(),
            dat_file_url
        ));
    }

    // ★★★ レスポンスボディをUTF-8文字列として取得 ★★★
    let content_str = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(format!(
                "レスポンス内容のテキスト取得に失敗しました (URL: {}): {}",
                dat_file_url, e
            ))
        }
    };

    // 以下の行のパース処理は、UTF-8文字列を前提としているため変更ありません
    let mut responses: Vec<ResponseItem> = Vec::new();
    for (index, line) in content_str.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        // splitn(5, "<>") を使用して、本文とタイトル(もしあれば)を分離
        let parts: Vec<&str> = line.splitn(5, "<>").collect();

        if parts.len() >= 4 {
            // 本文(parts[3])までは必須
            let name = parts[0].to_string();
            let mail = parts[1].to_string();
            let date_and_id_full = parts[2].to_string();
            let body = parts[3].to_string(); // parts[3] が純粋な本文

            // ... (日付とID情報のパース部分は変更なし) ...
            let mut date_str = date_and_id_full.clone();
            let mut id_info_str = "".to_string();
            if let Some(id_pos) = date_and_id_full.rfind(" ID:") {
                date_str = date_and_id_full[..id_pos].trim().to_string();
                id_info_str = date_and_id_full[id_pos..].trim().to_string();
            } else {
                if let Some(last_space_pos) = date_and_id_full.rfind(' ') {
                    let potential_date = date_and_id_full[..last_space_pos].trim();
                    if potential_date
                        .matches(|c: char| c == '/' || c == ':')
                        .count()
                        >= 4
                    {
                        date_str = potential_date.to_string();
                        id_info_str = date_and_id_full[last_space_pos..].trim().to_string();
                    }
                }
            }

            responses.push(ResponseItem {
                id: (index + 1).to_string(),
                author: name,
                mail,
                created_at: date_str,
                user_id_info: id_info_str,
                content: body,
            });
        } else {
            println!(
                "[Rust fetch_thread_content] 行のパースに失敗 (パーツ数 {}): {}",
                parts.len(),
                line
            );
        }
    }

    println!(
        "[Rust fetch_thread_content] {} 個のレスをパースしました (スレッドID: {})",
        responses.len(),
        thread_id
    );
    Ok(responses)
}
