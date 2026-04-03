use serde::{Deserialize, Serialize};
use tradewars_game::PlayerSnapshot;

// ── Client → Server Messages ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum ClientMessage {
    /// Player wants to join the world
    #[serde(rename = "join")]
    Join { name: String },

    /// Player movement delta
    #[serde(rename = "move")]
    Move { dx: f64, dz: f64 },

    /// Execute a trade at the nearest post
    #[serde(rename = "trade")]
    Trade {
        commodity_id: String,
        trade_type: String,
        quantity: u32,
    },

    /// Toggle trade panel open/close
    #[serde(rename = "toggle_trade_panel")]
    ToggleTradePanel,

    /// Close trade panel
    #[serde(rename = "close_trade_panel")]
    CloseTradePanel,
}

// ── Server → Client Messages ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Full game state snapshot (sent every tick + after mutations)
    #[serde(rename = "state")]
    State { snapshot: PlayerSnapshot },

    /// Response to a trade attempt
    #[serde(rename = "trade_result")]
    TradeResult { success: bool, message: String },

    /// Server assigned the client a player ID
    #[serde(rename = "welcome")]
    Welcome { player_id: String },

    /// Server error
    #[serde(rename = "error")]
    Error { message: String },
}
