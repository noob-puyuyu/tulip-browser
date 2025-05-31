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

let isRefreshingThreads = false;

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
async function displayThreadResponses(threadId, threadTitle) {
  // ... (既存のレスポンスクリア処理、タイトル表示処理は変更なし) ...
  if (!responseListElement || typeof invoke !== "function") {
    /* ...エラー処理... */ return;
  }
  responseListElement.innerHTML = "";
  mainContentPlaceholder.style.display = "none";
  currentThreadTitleElement.textContent = threadTitle;
  currentThreadTitleElement.style.display = "block";

  try {
    console.log(
      `[JS] Invoking 'fetch_thread_content' for threadId: ${threadId}`,
    );
    const responses = await invoke("fetch_thread_content", {
      threadId: threadId,
    });
    console.log("[JS] Responses received from Rust:", responses); // responses に新しいフィールドが含まれる

    if (responses && responses.length > 0) {
      responses.forEach((response) => {
        // response は拡張された ResponseItem 型
        const resItem = document.createElement("li");
        resItem.classList.add("response-item");

        const resHeader = document.createElement("div");
        resHeader.classList.add("response-header");

        const authorSpan = document.createElement("span");
        authorSpan.classList.add("response-author");
        authorSpan.textContent = response.author || "名無しさん";

        if (response.mail) {
          /* ...メールリンク作成... */
        }

        const dateSpan = document.createElement("span");
        dateSpan.classList.add("response-created-at");
        dateSpan.textContent = response.created_at;

        const idInfoSpan = document.createElement("span");
        idInfoSpan.classList.add("response-user-id"); // 必要ならCSSでスタイル調整
        let idDisplayText = response.user_id_info || "";
        // ★★★ IDカウンター表示の追加 ★★★
        if (response.parsed_user_id && response.id_total_count > 1) {
          // parsed_user_id があり、総投稿数が0より大きい場合
          idDisplayText += ` [${response.id_occurrence_count}/${response.id_total_count}]`;
        }
        idInfoSpan.textContent = idDisplayText.trim();

        resHeader.appendChild(authorSpan);
        resHeader.appendChild(dateSpan);
        if (idDisplayText) {
          // ID情報があれば表示
          resHeader.appendChild(idInfoSpan);
        }

        const resContent = document.createElement("div");
        resContent.classList.add("response-content");
        resContent.innerHTML = response.content;

        resItem.appendChild(resHeader);
        resItem.appendChild(resContent);
        responseListElement.appendChild(resItem);

        // ★★★ ここから追加: resContent内のImgur画像をプロキシ経由で読み込む ★★★
        const imagesInPost = resContent.querySelectorAll("img");
        imagesInPost.forEach(async (imgElement) => {
          const originalSrc = imgElement.getAttribute("src");
          // i.imgur.com の画像のみを対象とする (他のドメインは必要に応じて追加)
          if (originalSrc && originalSrc.startsWith("https://i.imgur.com/")) {
            console.log(
              "[JS] Imgur画像を発見、プロキシ経由で取得試行: ",
              originalSrc,
            );

            // 一時的にローディング画像などに差し替えるか、srcを空にする (任意)
            // imgElement.src = "path/to/loading.gif";
            const tempOriginalSrc = originalSrc; // エラー時に戻すためなどに保持

            try {
              const dataUrl = await invoke("fetch_image_as_base64", {
                url: tempOriginalSrc,
              });
              imgElement.src = dataUrl; // Base64データURIに置き換え
              console.log("[JS] 画像のプロキシ取得成功: ", tempOriginalSrc);
            } catch (error) {
              console.error(
                "[JS] 画像のプロキシ取得失敗",
                tempOriginalSrc,
                ":",
                error,
              );
              // エラーの場合、壊れた画像アイコンのままにするか、非表示にするか、
              // あるいは元のsrcに戻してブラウザに再試行させるか (403が再度発生する可能性)
              // imgElement.src = tempOriginalSrc; // ← これだと403が再発する可能性
              imgElement.alt = `画像読み込み失敗: ${tempOriginalSrc}`; // altテキスト設定
            }
          }
        });
        // ★★★ 追加ここまで ★★★
      });
    } else {
      /* ...レスなしの場合の処理... */
    }
  } catch (error) {
    /* ...エラー処理... */
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

// --- リサイズ機能のロジック ---
function initializeResizablePanels() {
  const container = document.querySelector(".container");
  const sidebar = document.getElementById("thread-list-panel");
  const resizer = document.getElementById("dragHandleX");
  // const mainContent = document.getElementById('main-content-area'); // 直接操作は不要な場合が多い

  if (!container || !sidebar || !resizer) {
    console.error("Resizable panel components not found. Resizing disabled.");
    return;
  }

  let isResizing = false;
  let initialSidebarWidth = 0;
  let startX = 0;

  resizer.addEventListener("mousedown", (e) => {
    isResizing = true;
    startX = e.clientX;
    initialSidebarWidth = sidebar.offsetWidth; // 現在の幅を取得

    // ドラッグ中のテキスト選択や他のマウスイベントを一時的に無効化
    document.body.style.userSelect = "none";
    document.body.style.pointerEvents = "none"; // 他の要素のマウスイベントをブロック

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    console.log("Resizer mousedown: Start resizing", {
      startX,
      initialSidebarWidth,
    });
  });

  function handleMouseMove(e) {
    if (!isResizing) return;

    const deltaX = e.clientX - startX;
    let newWidth = initialSidebarWidth + deltaX;

    // CSSで設定された最小幅・最大幅を取得して制約をかける
    const computedStyle = getComputedStyle(sidebar);
    const minWidth = parseInt(computedStyle.minWidth, 10) || 0;
    // maxWidth は % 指定の場合、コンテナ幅から計算する必要がある
    let maxWidth = Infinity;
    if (computedStyle.maxWidth && computedStyle.maxWidth.endsWith("%")) {
      const percentage = parseFloat(computedStyle.maxWidth) / 100;
      maxWidth = container.offsetWidth * percentage;
    } else if (computedStyle.maxWidth && computedStyle.maxWidth !== "none") {
      maxWidth = parseInt(computedStyle.maxWidth, 10);
    }

    if (newWidth < minWidth) {
      newWidth = minWidth;
    }
    if (newWidth > maxWidth) {
      newWidth = maxWidth;
    }

    sidebar.style.flexBasis = `${newWidth}px`; // flex-basis を更新
    // sidebar.style.width = `${newWidth}px`; // width でも可だが flex-basis の方が flex 環境では適切
    console.log("Resizing, new width:", newWidth);
  }

  function handleMouseUp() {
    if (!isResizing) return;
    isResizing = false;

    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);

    // 無効化したスタイルを元に戻す
    document.body.style.userSelect = "";
    document.body.style.pointerEvents = "";
    console.log(
      "Resizer mouseup: End resizing. Final width:",
      sidebar.style.flexBasis,
    );
    // TODO: 必要であれば、ここでリサイズ後の幅をlocalStorageなどに保存する
  }
  console.log("Resizable panels initialized.");
}

function setupThreadListRefresh() {
  const sidebarPanel = document.getElementById("thread-list-panel"); // スクロール可能なサイドバーパネル

  if (!sidebarPanel) {
    console.error(
      "[JS] スレッド一覧パネル (#thread-list-panel) が見つかりません。更新機能は無効です。",
    );
    return;
  }

  sidebarPanel.addEventListener(
    "wheel",
    async (event) => {
      // event.deltaY < 0 は上方向へのホイール操作
      // sidebarPanel.scrollTop === 0 はスクロールが一番上にある状態

      if (isRefreshingThreads) {
        // console.log("[JS] 現在スレッド一覧を更新中です。");
        event.preventDefault(); // 多重更新を防ぎつつ、意図しないスクロールも抑制
        return;
      }

      if (sidebarPanel.scrollTop === 0 && event.deltaY < 0) {
        console.log(
          "[JS] スレッド一覧の最上部で上スクロールを検知しました。一覧を更新します。",
        );
        event.preventDefault(); // デフォルトのスクロール動作（ページ全体のスクロールなど）をキャンセル

        isRefreshingThreads = true;

        // --- 更新中インジケーター表示 (簡易版) ---
        let refreshIndicator = document.getElementById("refresh-indicator");
        if (!refreshIndicator) {
          refreshIndicator = document.createElement("div");
          refreshIndicator.id = "refresh-indicator";
          refreshIndicator.textContent = "スレッド一覧を更新中...";
          refreshIndicator.style.padding = "10px";
          refreshIndicator.style.textAlign = "center";
          refreshIndicator.style.backgroundColor = "#f0f0f0"; // 仮の背景色
          // スレッドリストの先頭に追加
          if (threadListElement.firstChild) {
            threadListElement.insertBefore(
              refreshIndicator,
              threadListElement.firstChild,
            );
          } else {
            threadListElement.appendChild(refreshIndicator);
          }
        }
        refreshIndicator.style.display = "block";
        // ------------------------------------

        try {
          await loadAndDisplayThreads(); // 既存のスレッド読み込み・表示関数を呼び出す
          console.log("[JS] スレッド一覧が正常に更新されました。");
        } catch (error) {
          console.error("[JS] スレッド一覧の更新に失敗しました:", error);
          // loadAndDisplayThreads 内でエラー表示されるはずなので、ここでは特に何もしないか、
          // または追加のエラーフィードバックを表示
        } finally {
          // --- 更新中インジケーター非表示 ---
          if (refreshIndicator) {
            refreshIndicator.style.display = "none"; // または removeChild(refreshIndicator)
          }
          // --------------------------------
          isRefreshingThreads = false; // 更新フラグをリセット
        }
      }
    },
    { passive: false },
  ); // preventDefault を呼ぶために passive: false を指定

  console.log(
    "[JS] スレッド一覧の更新機能（スクロールアップ）が初期化されました。",
  );
}

// DOMが読み込まれたらスレッドを読み込む
document.addEventListener("DOMContentLoaded", () => {
  console.log("[JS] DOMContentLoaded event fired.");
  loadAndDisplayThreads();
  initializeResizablePanels();
  setupThreadListRefresh();
});
