// Tauri APIをインポート
// const { invoke } = window.__TAURI__.tauri; // 古いTauri v1の場合
// 以下のいずれかがTauri v1.xの比較的新しいバージョンやv2で動作する可能性があります
// const { invoke } = window.__TAURI__; // Tauri v2 alpha や一部のv1
const { invoke } = window.__TAURI__.core; // 以前のやり取りでこちらが機能した可能性

// HTML要素の取得
const threadListElement = document.getElementById('thread-list');
const mainContentArea = document.getElementById('main-content-area');
const mainContentPlaceholder = document.getElementById('main-content-placeholder');
const currentThreadTitleElement = document.getElementById('current-thread-title');
const responseListElement = document.getElementById('response-list');

// スレッドアイテムをDOMに追加する関数 (変更なし)
function addThreadToView(thread) {
  const listItem = document.createElement('li');
  listItem.classList.add('thread-item');
  listItem.dataset.threadId = thread.id;
  listItem.dataset.threadTitle = thread.title; // タイトルもデータ属性として保持

  const header = document.createElement('div');
  header.classList.add('thread-header');
  header.textContent = thread.created_at;

  const content = document.createElement('div');
  content.classList.add('thread-content');

  const title = document.createElement('span');
  title.classList.add('thread-title');
  title.textContent = thread.title;
  title.title = thread.title;

  const responseCount = document.createElement('span');
  responseCount.classList.add('thread-response-count');
  responseCount.textContent = `${thread.response_count}レス`;

  content.appendChild(title);
  content.appendChild(responseCount);

  listItem.appendChild(header);
  listItem.appendChild(content);

  // クリックイベント: スレッドのレスポンスを表示
  listItem.addEventListener('click', () => {
    const threadId = listItem.dataset.threadId;
    const threadTitle = listItem.dataset.threadTitle;
    console.log(`Thread clicked: ${threadId} - ${threadTitle}`);
    displayThreadResponses(threadId, threadTitle);

    // クリックされたスレッドを視覚的に示す（任意）
    document.querySelectorAll('#thread-list .thread-item').forEach(item => {
        item.classList.remove('active');
    });
    listItem.classList.add('active');
  });

  threadListElement.appendChild(listItem);
}

// 特定スレッドのレスポンスをメインコンテンツエリアに表示する関数
async function displayThreadResponses(threadId, threadTitle) {
  if (typeof invoke !== 'function') {
    console.error("[JS] Error: 'invoke' is not a function or not defined!");
    responseListElement.innerHTML = '<li>アプリケーションの初期化に問題があります。</li>';
    mainContentPlaceholder.style.display = 'none';
    currentThreadTitleElement.style.display = 'none';
    return;
  }

  // 以前のレスポンスをクリア
  responseListElement.innerHTML = '';
  // 初期メッセージを隠し、タイトルを表示
  mainContentPlaceholder.style.display = 'none';
  currentThreadTitleElement.textContent = threadTitle;
  currentThreadTitleElement.style.display = 'block';

  try {
    console.log(`[JS] Invoking 'get_dummy_responses' for threadId: ${threadId}`);
    const responses = await invoke('get_dummy_responses', { threadId: threadId });
    console.log("[JS] Responses received from Rust:", responses);

    if (responses && responses.length > 0) {
      responses.forEach(response => {
        const resItem = document.createElement('li');
        resItem.classList.add('response-item');

        const resHeader = document.createElement('div');
        resHeader.classList.add('response-header');

        const authorSpan = document.createElement('span');
        authorSpan.classList.add('response-author');
        authorSpan.textContent = response.author || "名無しさん"; // authorがない場合のフォールバック

        const createdAtSpan = document.createElement('span');
        createdAtSpan.classList.add('response-created-at');
        createdAtSpan.textContent = response.created_at;

        resHeader.appendChild(authorSpan);
        resHeader.appendChild(createdAtSpan);

        const resContent = document.createElement('div');
        resContent.classList.add('response-content');
        resContent.textContent = response.content;

        resItem.appendChild(resHeader);
        resItem.appendChild(resContent);
        responseListElement.appendChild(resItem);
      });
    } else {
      responseListElement.innerHTML = '<li>このスレッドにはまだレスポンスがありません。</li>';
    }
  } catch (error) {
    console.error(`[JS] スレッド (${threadId}) のレスポンス読み込みに失敗しました:`, error);
    responseListElement.innerHTML = `<li>レスポンスの読み込みに失敗しました。<br>${error}</li>`;
    // エラー時もタイトルは表示したままにするか、初期状態に戻すか選択
    // currentThreadTitleElement.style.display = 'none';
    // mainContentPlaceholder.style.display = 'block';
  }
}


// ダミースレッド一覧を読み込んで表示する関数 (変更なし)
async function loadAndDisplayThreads() {
  console.log("[JS] loadAndDisplayThreads called");
  if (!threadListElement) {
    console.error("[JS] Error: threadListElement is not found!");
    return;
  }
   if (typeof invoke !== 'function') {
    console.error("[JS] Error: 'invoke' is not a function or not defined during loadAndDisplayThreads!");
    threadListElement.innerHTML = '<li>アプリケーションの初期化に問題があります。</li>';
    return;
  }
  try {
    console.log("[JS] Invoking 'get_dummy_threads'...");
    const dummyThreads = await invoke('get_dummy_threads');
    console.log("[JS] Threads received from Rust:", dummyThreads);

    if (dummyThreads && dummyThreads.length > 0) {
      threadListElement.innerHTML = ''; // 既存の項目をクリア
      dummyThreads.forEach(addThreadToView);
    } else {
      threadListElement.innerHTML = '<li>スレッドがありません。</li>';
    }
  } catch (error) {
    console.error('[JS] スレッドの読み込みに失敗しました:', error);
    threadListElement.innerHTML = '<li>スレッドの読み込みに失敗しました。</li>';
  }
}

// DOMが読み込まれたらスレッドを読み込む
document.addEventListener('DOMContentLoaded', () => {
  console.log("[JS] DOMContentLoaded event fired.");
  loadAndDisplayThreads();
});