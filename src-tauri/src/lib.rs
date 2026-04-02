pub mod market;

use market::{GameState, TradeResult};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub game: Mutex<GameState>,
}

// ── Tick loop (runs in background thread) ─────────────────────

fn start_tick_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let tick_base = Duration::from_secs(1); // 1 second per tick at x1
        let mut last_tick = Instant::now();

        loop {
            std::thread::sleep(Duration::from_millis(16)); // ~60fps check rate

            let state_handle = app.state::<AppState>();
            let mut game = state_handle.game.lock().unwrap();

            if game.paused {
                last_tick = Instant::now();
                continue;
            }

            let tick_interval = tick_base.div_f64(game.speed.max(0.1));
            if last_tick.elapsed() >= tick_interval {
                market::advance_tick(&mut game);
                last_tick = Instant::now();

                // Emit full state to frontend
                let snapshot = game.clone();
                drop(game); // release lock before emit
                let _ = app.emit("game-state", &snapshot);
            }
        }
    });
}

// ── Tauri Commands ────────────────────────────────────────────

#[tauri::command]
fn get_game_state(state: State<AppState>) -> Result<GameState, String> {
    let game = state.game.lock().map_err(|e| e.to_string())?;
    Ok(game.clone())
}

#[tauri::command]
fn move_player(state: State<AppState>, app: AppHandle, dx: f64, dz: f64) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    market::move_player(&mut game, dx, dz);
    let snapshot = game.clone();
    drop(game);
    let _ = app.emit("game-state", &snapshot);
    Ok(())
}

#[tauri::command]
fn execute_trade(state: State<AppState>, app: AppHandle, commodity_id: String, trade_type: String, quantity: u32) -> Result<TradeResult, String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    let result = market::execute_trade(&mut game, &commodity_id, &trade_type, quantity);
    if result.success {
        game.notification = result.message.clone();
    }
    let snapshot = game.clone();
    drop(game);
    let _ = app.emit("game-state", &snapshot);
    Ok(result)
}

#[tauri::command]
fn toggle_trade_panel(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    market::toggle_trade_panel(&mut game);
    let snapshot = game.clone();
    drop(game);
    let _ = app.emit("game-state", &snapshot);
    Ok(())
}

#[tauri::command]
fn set_paused(state: State<AppState>, paused: bool) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    market::set_paused(&mut game, paused);
    Ok(())
}

#[tauri::command]
fn set_speed(state: State<AppState>, speed: f64) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    market::set_speed(&mut game, speed);
    Ok(())
}

#[tauri::command]
fn close_trade_panel(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    game.show_trade_panel = false;
    let snapshot = game.clone();
    drop(game);
    let _ = app.emit("game-state", &snapshot);
    Ok(())
}

#[tauri::command]
fn reset_game(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    *game = market::create_initial_state();
    let snapshot = game.clone();
    drop(game);
    let _ = app.emit("game-state", &snapshot);
    Ok(())
}

// ── App Entry ─────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            game: Mutex::new(market::create_initial_state()),
        })
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            move_player,
            execute_trade,
            toggle_trade_panel,
            set_paused,
            set_speed,
            close_trade_panel,
            reset_game
        ])
        .setup(|app| {
            start_tick_loop(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
