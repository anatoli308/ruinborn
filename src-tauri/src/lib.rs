// ── Tauri Desktop Client ──────────────────────────────────────
// Thin wrapper: opens a window that loads the web client.
// All game logic runs on the dedicated server (tradewars-server).
// The frontend connects via WebSocket — no IPC commands needed.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
