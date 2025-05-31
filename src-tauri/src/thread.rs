use base64::{engine::general_purpose::STANDARD as Base64Standard, Engine as _};
use chrono::DateTime;
use chrono_tz::Asia::Tokyo;
use html_escape::decode_html_entities;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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

// レスポンスアイテムの構造体
#[derive(Debug, Serialize, Clone)] // フロントエンドに渡すので Serialize は必須
pub struct ResponseItem {
    id: String,           // レス番号 (例: "1", "2")
    author: String,       // 名前欄
    mail: String,         // メール欄
    created_at: String,   // パースされた日付部分の文字列
    user_id_info: String, // "ID:xxxx主" のような、表示用のID文字列全体
    content: String,      // 本文 (HTMLが含まれる)

    // ★★★ IDカウンター用に新しいフィールドを追加 ★★★
    parsed_user_id: Option<String>, // パースされた実際のID部分 (例: "R780OCsAQ")、IDがない場合は None
    id_occurrence_count: u32,       // このレスが、このIDによる何回目の投稿か
    id_total_count: u32,            // このIDがこのスレッドで行った総投稿数
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
        /* ... エラー処理 ... */
        return Err("スレッドIDが指定されていません。".to_string());
    }
    let dir_prefix = if thread_id.len() >= 4 {
        &thread_id[0..4]
    } else {
        /* ... エラー処理 ... */
        return Err("スレッドIDが短すぎます".to_string());
    };
    let dat_file_url = format!(
        /* ... URL生成 ... */
        "https://tulipplantation.com/tulipplantation/thread/{}/{}.dat",
        dir_prefix, thread_id
    );
    println!(
        "[Rust fetch_thread_content] スレッド内容を取得します (ID: {}): {}",
        thread_id, dat_file_url
    );

    let client = reqwest::Client::new();
    let response = match client.get(&dat_file_url).send().await {
        /* ... HTTP GET ... */ Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    if !response.status().is_success() {
        /* ... HTTPエラー処理 ... */
        return Err(response.status().to_string());
    }
    let content_str = match response.text().await {
        /* ... UTF-8としてテキスト取得 ... */ Ok(t) => t,
        Err(e) => return Err(e.to_string()),
    };

    let mut temp_responses: Vec<TempResponseData> = Vec::new();
    let mut id_total_counts: HashMap<String, u32> = HashMap::new();

    // 1回目のパース: 基本情報抽出とIDの総出現回数のカウント
    for line in content_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(5, "<>").collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let mail = parts[1].to_string();
            let date_and_id_full = parts[2].to_string();
            let body = parts[3].to_string();

            let mut date_str = date_and_id_full.clone();
            let mut user_id_info_str = "".to_string();
            if let Some(id_pos) = date_and_id_full.rfind(" ID:") {
                date_str = date_and_id_full[..id_pos].trim().to_string();
                user_id_info_str = date_and_id_full[id_pos..].trim().to_string();
            } else { /* ... 他の heuristic ... */
            }

            let parsed_id = parse_actual_id_from_info_str(&user_id_info_str);
            if let Some(ref id) = parsed_id {
                *id_total_counts.entry(id.clone()).or_insert(0) += 1;
            }

            temp_responses.push(TempResponseData {
                name,
                mail,
                date_str,
                user_id_info: user_id_info_str,
                parsed_user_id: parsed_id,
                body,
            });
        } else { /* ... パース失敗ログ ... */
        }
    }

    let mut final_responses: Vec<ResponseItem> = Vec::new();
    let mut id_current_occurrences: HashMap<String, u32> = HashMap::new();

    // 2回目の処理: ResponseItem の作成と、IDの現在の出現回数のカウント
    for (index, temp_res) in temp_responses.into_iter().enumerate() {
        let mut occurrence = 0;
        let mut total = 0;

        if let Some(ref parsed_id_val) = temp_res.parsed_user_id {
            let current_count_for_id = id_current_occurrences
                .entry(parsed_id_val.clone())
                .or_insert(0);
            *current_count_for_id += 1;
            occurrence = *current_count_for_id;
            total = *id_total_counts.get(parsed_id_val).unwrap_or(&0); // total_counts には必ずあるはず
        }

        final_responses.push(ResponseItem {
            id: (index + 1).to_string(),
            author: temp_res.name,
            mail: temp_res.mail,
            created_at: temp_res.date_str,
            user_id_info: temp_res.user_id_info,
            content: temp_res.body,
            parsed_user_id: temp_res.parsed_user_id, // これも渡す
            id_occurrence_count: occurrence,
            id_total_count: total,
        });
    }

    println!(
        "[Rust fetch_thread_content] {} 個のレスを処理完了 (スレッドID: {})",
        final_responses.len(),
        thread_id
    );
    Ok(final_responses)
}

#[tauri::command]
pub async fn fetch_image_as_base64(url: String) -> Result<String, String> {
    println!("[Rust fetch_image_as_base64] 画像を取得します: {}", url);

    let client = reqwest::Client::new();
    let response = match client
        .get(&url)
        // Imgurが特定のUser-Agentを要求する可能性は低いですが、念のため一般的なものを設定するのも一手
        // .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let err_msg = format!("画像リクエストに失敗しました (URL: {}): {}", url, e);
            eprintln!("[Rust fetch_image_as_base64] {}", err_msg);
            return Err(err_msg);
        }
    };

    if !response.status().is_success() {
        let err_msg = format!(
            "画像リクエストでHTTPエラー {} (URL: {})",
            response.status(),
            url
        );
        eprintln!("[Rust fetch_image_as_base64] {}", err_msg);
        return Err(err_msg);
    }

    // Content-TypeヘッダーからMIMEタイプを取得 (例: "image/jpeg", "image/png")
    // 取得できない場合のフォールバックとして "image/jpeg" を使用
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/jpeg") // デフォルト、またはURLの拡張子から判定するロジックを追加しても良い
        .to_string();

    let image_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            let err_msg = format!("画像データのバイト取得に失敗しました (URL: {}): {}", url, e);
            eprintln!("[Rust fetch_image_as_base64] {}", err_msg);
            return Err(err_msg);
        }
    };

    // Base64エンコード
    let base64_encoded = Base64Standard.encode(&image_bytes);
    let data_url = format!("data:{};base64,{}", content_type, base64_encoded);

    Ok(data_url)
}

fn parse_actual_id_from_info_str(user_id_info_str: &str) -> Option<String> {
    if let Some(id_start_idx) = user_id_info_str.find("ID:") {
        let after_id_colon = &user_id_info_str[id_start_idx + 3..];
        // ID部分の終わりを見つける (スペース、(、[ などが区切りになることが多い)
        let id_end_idx = after_id_colon
            .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .unwrap_or(after_id_colon.len());
        let actual_id = after_id_colon[..id_end_idx].to_string();
        if !actual_id.is_empty() {
            return Some(actual_id);
        }
    }
    None
}

#[derive(Debug)]
struct TempResponseData {
    name: String,
    mail: String,
    date_str: String,               // パースされた日付部分
    user_id_info: String,           // IDなどを含む文字列全体
    parsed_user_id: Option<String>, // パースされた実際のID
    body: String,
}
