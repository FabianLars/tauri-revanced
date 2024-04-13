window.print = function () {
    return window.__TAURI_INTERNALS__.invoke('plugin:webview|print');
};
