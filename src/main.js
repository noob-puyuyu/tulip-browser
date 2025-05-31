// Tauri APIをインポート
// const { invoke } = window.__TAURI__.tauri; // 古いTauri v1の場合
// 以下のいずれかがTauri v1.xの比較的新しいバージョンやv2で動作する可能性があります
// const { invoke } = window.__TAURI__; // Tauri v2 alpha や一部のv1
const { invoke } = window.__TAURI__.core; // 以前のやり取りでこちらが機能した可能性

// HTML要素の取得
const threadListElement = document.getElementById("thread-list");
const mainContentArea = document.getElementById("main-content-area");
const mainContentPlaceholder = document.getElementById(
  "main-content-placeholder",
);
const currentThreadTitleElement = document.getElementById(
  "current-thread-title",
);
const responseListElement = document.getElementById("response-list");

// スレッドアイテムをDOMに追加する関数 (変更なし)
function addThreadToView(thread) {
  const listItem = document.createElement("li");
  listItem.classList.add("thread-item");
  listItem.dataset.threadId = thread.id;
  listItem.dataset.threadTitle = thread.title; // タイトルもデータ属性として保持

  const header = document.createElement("div");
  header.classList.add("thread-header");
  header.textContent = thread.created_at;

  const content = document.createElement("div");
  content.classList.add("thread-content");

  const title = document.createElement("span");
  title.classList.add("thread-title");
  title.textContent = thread.title;
  title.title = thread.title;

  const responseCount = document.createElement("span");
  responseCount.classList.add("thread-response-count");
  responseCount.textContent = `${thread.response_count}レス`;

  content.appendChild(title);
  content.appendChild(responseCount);

  listItem.appendChild(header);
  listItem.appendChild(content);

  // クリックイベント: スレッドのレスポンスを表示
  listItem.addEventListener("click", () => {
    const threadId = listItem.dataset.threadId;
    const threadTitle = listItem.dataset.threadTitle;
    console.log(`Thread clicked: ${threadId} - ${threadTitle}`);
    displayThreadResponses(threadId, threadTitle);

    // クリックされたスレッドを視覚的に示す（任意）
    document.querySelectorAll("#thread-list .thread-item").forEach((item) => {
      item.classList.remove("active");
    });
    listItem.classList.add("active");
  });

  threadListElement.appendChild(listItem);
}

// 特定スレッドのレスポンスをメインコンテンツエリアに表示する関数
// 特定スレッドのレスポンスをメインコンテンツエリアに表示する関数を修正
async function displayThreadResponses(threadId, threadTitle) {
  if (typeof invoke !== "function") {
    console.error("[JS] Error: 'invoke' is not a function or not defined!");
    // ... (エラー表示処理) ...
    return;
  }

  // 以前のレスポンスをクリアし、タイトルを表示
  responseListElement.innerHTML = "";
  mainContentPlaceholder.style.display = "none";
  currentThreadTitleElement.textContent = threadTitle;
  currentThreadTitleElement.style.display = "block";

  try {
    console.log(
      `[JS] Invoking 'fetch_thread_content' for threadId: ${threadId}`,
    );
    // ★★★ コマンド名と引数を変更 ★★★
    const responses = await invoke("fetch_thread_content", {
      threadId: threadId,
    });
    console.log("[JS] Responses received from Rust:", responses);

    if (responses && responses.length > 0) {
      responses.forEach((response) => {
        // response は ResponseItem 型のオブジェクト
        const resItem = document.createElement("li");
        resItem.classList.add("response-item");

        const resHeader = document.createElement("div");
        resHeader.classList.add("response-header");

        const authorSpan = document.createElement("span");
        authorSpan.classList.add("response-author");
        authorSpan.textContent = response.author || "名無しさん";

        // メール欄を表示する場合 (任意)
        if (response.mail) {
          const mailLink = document.createElement("a");
          mailLink.href = `mailto:${response.mail}`;
          mailLink.textContent = `[${response.mail}]`;
          mailLink.style.marginLeft = "5px"; // 適当なスタイル
          authorSpan.appendChild(mailLink);
        }

        const dateAndIdSpan = document.createElement("span");
        dateAndIdSpan.classList.add("response-created-at"); // CSSクラス名はそのまま利用
        // created_at には日付部分、user_id_info にはID部分が入る想定
        dateAndIdSpan.textContent =
          `${response.created_at} ${response.user_id_info || ""}`.trim();

        resHeader.appendChild(authorSpan);
        resHeader.appendChild(dateAndIdSpan);

        const resContent = document.createElement("div");
        resContent.classList.add("response-content");
        // 本文はHTMLとして解釈・挿入する (サニタイズが必要な場合は別途処理)
        resContent.innerHTML = response.content; // ★★★ .textContent から .innerHTML に変更 ★★★

        resItem.appendChild(resHeader);
        resItem.appendChild(resContent);
        responseListElement.appendChild(resItem);
      });
    } else {
      responseListElement.innerHTML =
        "<li>このスレッドにはレスポンスがありません。</li>";
    }
  } catch (error) {
    console.error(
      `[JS] スレッド (${threadId}) のレスポンス読み込みに失敗しました:`,
      error,
    );
    responseListElement.innerHTML = `<li>レスポンスの読み込みに失敗しました。<br>${error}</li>`;
  }
}

async function loadAndDisplayThreads() {
  console.log("[JS] loadAndDisplayThreads called");
  if (!threadListElement || typeof invoke !== "function") {
    console.error(
      "[JS] Error: threadListElement or invoke function is not available!",
    );
    threadListElement.innerHTML = "<li>初期化エラーが発生しました。</li>";
    return;
  }

  try {
    console.log("[JS] Invoking 'fetch_threads' to get thread list from URL...");
    const threads = await invoke("fetch_threads"); // ★★★ 引数を削除 ★★★
    console.log("[JS] Threads received from Rust (fetch_threads):", threads);

    if (threads && threads.length > 0) {
      threadListElement.innerHTML = ""; // 既存の項目をクリア
      threads.forEach(addThreadToView); // addThreadToView関数は既存のものをそのまま使用
    } else {
      threadListElement.innerHTML = "<li>表示できるスレッドがありません。</li>";
    }
  } catch (error) {
    console.error(
      "[JS] スレッドの読み込みに失敗しました (fetch_threads):",
      error,
    );
    threadListElement.innerHTML = `<li>スレッドの読み込みに失敗しました。<br>エラー: ${error}</li>`;
  }
}

// DOMが読み込まれたらスレッドを読み込む
document.addEventListener("DOMContentLoaded", () => {
  console.log("[JS] DOMContentLoaded event fired.");
  loadAndDisplayThreads();
});
