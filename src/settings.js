// settings.js
const { invoke } = window.__TAURI__.core; // 以前のやり取りでこちらが機能した可能性
const { getCurrentWindow } = window.__TAURI__.window; // 以前のやり取りでこちらが機能した可能性

const themeSelect = document.getElementById("theme-select");
const fontSizeInput = document.getElementById("font-size-input");
const saveButton = document.getElementById("save-button");
// HTML要素が正しく取得できているか確認
console.log("settings.js: themeSelect 要素:", themeSelect);
console.log("settings.js: fontSizeInput 要素:", fontSizeInput);
console.log("settings.js: saveButton 要素:", saveButton);

// 設定を読み込んでフォームに反映する関数
async function loadSettings() {
  console.log("settings.js: loadSettings() 関数呼び出し");
  try {
    const settings = await invoke("get_settings");
    console.log("settings.js: 読み込んだ設定:", settings);
    if (themeSelect && settings) {
      // 要素が存在するか確認
      themeSelect.value = settings.theme || "light";
    }
    if (fontSizeInput && settings) {
      // 要素が存在するか確認
      fontSizeInput.value = settings.font_size || 14;
    }
  } catch (error) {
    console.error("settings.js: 設定の読み込みに失敗:", error);
    if (themeSelect) themeSelect.value = "light"; // フォールバック
    if (fontSizeInput) fontSizeInput.value = 14; // フォールバック
  }
}

// 現在のフォームの値から設定を保存する関数
async function saveSettings() {
  console.log("settings.js: saveSettings() 関数呼び出し"); // ★ボタンクリックでこれが表示されるか？

  if (!themeSelect || !fontSizeInput) {
    console.error("settings.js: 設定用フォーム要素が見つかりません。");
    alert("エラー: 設定用フォーム要素が見つかりません。");
    return;
  }

  const newSettings = {
    theme: themeSelect.value,
    font_size: parseInt(fontSizeInput.value, 10),
  };
  console.log("settings.js: 保存する新しい設定:", newSettings);

  try {
    console.log("settings.js: Rustコマンド 'save_settings' を呼び出します...");
    await invoke("save_settings", { settings: newSettings });
    console.log("settings.js: Rustコマンド成功。設定が保存されました。");

    // ユーザーに保存完了を通知 (任意)
    alert("設定が保存されました！");

    console.log("settings.js: 設定ウィンドウを閉じます...");
    const appWindow = getCurrentWindow();
    await appWindow.close();
    console.log("settings.js: 設定ウィンドウが閉じられました。");
  } catch (error) {
    console.error("settings.js: 設定の保存に失敗:", error);
    alert("設定の保存に失敗しました: " + error.message); // error.message で詳細表示
  }
}

// 保存ボタンにイベントリスナーを登録
if (saveButton) {
  saveButton.addEventListener("click", saveSettings);
  console.log("settings.js: 保存ボタンにイベントリスナーを登録しました。");
} else {
  console.error(
    "settings.js: 保存ボタンが見つかりません。イベントリスナーを登録できませんでした。",
  );
}

// DOMが読み込まれたら設定をロード
document.addEventListener("DOMContentLoaded", () => {
  console.log("settings.js: DOMContentLoaded イベント発生");
  loadSettings();
});
