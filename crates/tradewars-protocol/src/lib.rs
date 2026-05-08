use serde::{Deserialize, Serialize};
use tradewars_game::{DeltaSnapshot, PlayerSnapshot};

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

    /// Gather from nearest resource node
    #[serde(rename = "gather")]
    Gather,

    /// Create a player-owned market at current position
    #[serde(rename = "create_market")]
    CreateMarket { name: String },

    /// Post a buy/sell order on own market
    #[serde(rename = "post_order")]
    PostOrder {
        commodity_id: String,
        order_type: String,
        quantity: u32,
        price_per_unit: f64,
    },

    /// Cancel an order on own market
    #[serde(rename = "cancel_order")]
    CancelOrder { order_id: String },

    /// Fill an order at another player's market
    #[serde(rename = "fill_order")]
    FillOrder {
        market_id: String,
        order_id: String,
        quantity: u32,
    },

    /// Accept a mission from the board
    #[serde(rename = "accept_mission")]
    AcceptMission { mission_id: String },

    /// Toggle trade panel open/close
    #[serde(rename = "toggle_trade_panel")]
    ToggleTradePanel,

    /// Close trade panel
    #[serde(rename = "close_trade_panel")]
    CloseTradePanel,

    /// Move/swap an item between two bag slots
    #[serde(rename = "move_item")]
    MoveItem {
        src_bag: u32,
        src_slot: u32,
        dst_bag: u32,
        dst_slot: u32,
    },

    /// Permanently drop an item from a bag slot
    #[serde(rename = "drop_item")]
    DropItem { bag: u32, slot: u32 },

    /// Bind an item to an action-bar slot (item_id = None to clear)
    #[serde(rename = "set_action_slot")]
    SetActionSlot { slot: u32, item_id: Option<String> },
    #[serde(rename = "set_action_slot_skill")]
    SetActionSlotSkill { slot: u32, skill_id: String },

    /// Trigger an action-bar slot (1-9 hotkeys)
    #[serde(rename = "use_action_slot")]
    UseActionSlot { slot: u32 },

    /// Equip an item from a bag slot into a paperdoll slot.
    /// `target` = None means: auto-pick by item type.
    #[serde(rename = "equip_item")]
    EquipItem {
        bag: u32,
        slot: u32,
        target: Option<String>,
    },

    /// Unequip a paperdoll slot back into the bags.
    #[serde(rename = "unequip_item")]
    UnequipItem { target: String },

    // ── D2 Combat ──
    /// Player attacks an enemy (left/right click). `mouse_button`: 0 = left, 1 = right.
    #[serde(rename = "attack")]
    Attack { enemy_id: String, mouse_button: u8 },

    /// Pick up a loot drop on the ground.
    #[serde(rename = "pickup_loot")]
    PickupLoot { loot_id: String },

    /// Travel to a known waypoint zone.
    #[serde(rename = "travel_waypoint")]
    TravelWaypoint { zone: String },

    /// Spend one unspent stat point.
    #[serde(rename = "allocate_stat")]
    AllocateStat { stat: String },

    /// Bind a mouse button to an item or basic attack (item_id = None -> basic attack).
    #[serde(rename = "set_mouse_skill")]
    SetMouseSkill { mouse_button: u8, item_id: Option<String> },

    /// Bind a mouse button to a skill (D2-style). skill_id = None -> basic attack.
    #[serde(rename = "bind_mouse_skill")]
    BindMouseSkill { mouse_button: u8, skill_id: Option<String> },

    /// First-time class pick (Barbarian / Sorceress / Necromancer).
    #[serde(rename = "choose_class")]
    ChooseClass { class: String },

    /// Spend one unspent skill point on a skill.
    #[serde(rename = "allocate_skill")]
    AllocateSkill { skill_id: String },

    /// Cast an active skill. Targets are optional and effect-specific.
    #[serde(rename = "cast_skill")]
    CastSkill {
        skill_id: String,
        target_enemy_id: Option<String>,
        target_x: Option<f64>,
        target_z: Option<f64>,
    },
}

// ── Server → Client Messages ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Full game state snapshot (sent on join + after major actions)
    #[serde(rename = "state")]
    State { snapshot: PlayerSnapshot },

    /// Delta update — only changed data (sent at 20 Hz tick broadcast)
    #[serde(rename = "delta")]
    Delta { snapshot: DeltaSnapshot },

    /// Response to a player action
    #[serde(rename = "action_result")]
    ActionResult { success: bool, message: String },

    /// Server assigned the client a player ID
    #[serde(rename = "welcome")]
    Welcome { player_id: String },

    /// Server error
    #[serde(rename = "error")]
    Error { message: String },
}
