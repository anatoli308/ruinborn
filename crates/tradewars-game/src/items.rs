//! Items, Affixes, Bags und Action Bar — Diablo-2-inspirierte Random-Loot-Logik.
//!
//! Items unterscheiden sich klar von Commodities: jedes Item ist eine
//! eindeutige Instanz mit eigenem RNG-Roll. Commodities bleiben weiterhin
//! stackbare Ressourcen (HashMap<commodity_id, u32>).

use rand::Rng;
use serde::{Deserialize, Serialize};

// ── Konstanten ────────────────────────────────────────────────

/// Anzahl Bag-Slots am unteren rechten Rand (wie in WoW: 1 Backpack + 4 Equip-Slots).
pub const BAG_SLOT_COUNT: usize = 5;
/// Slot-Kapazität des Default-Backpacks (Bag 0). Nicht entfernbar.
pub const DEFAULT_BACKPACK_CAPACITY: u32 = 16;
/// Anzahl Action-Bar-Slots (1-9 Hotkeys).
pub const ACTION_BAR_SLOTS: usize = 9;
/// Drop-Wahrscheinlichkeit beim Sammeln (0.0..1.0).
pub const ITEM_DROP_CHANCE: f64 = 0.08;
/// Maximale Anzahl Affixe pro Item.
pub const MAX_AFFIXES: usize = 6;

// ── Rarity ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Magic,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    /// Roll a rarity weighted toward common.
    pub fn roll(rng: &mut impl Rng) -> Self {
        let r: f64 = rng.gen();
        match r {
            x if x < 0.55 => Rarity::Common,
            x if x < 0.85 => Rarity::Magic,
            x if x < 0.97 => Rarity::Rare,
            x if x < 0.995 => Rarity::Epic,
            _ => Rarity::Legendary,
        }
    }

    /// Wieviele Affixe rollen?
    fn affix_count(self, rng: &mut impl Rng) -> usize {
        match self {
            Rarity::Common => 0,
            Rarity::Magic => rng.gen_range(1..=2),
            Rarity::Rare => rng.gen_range(3..=4),
            Rarity::Epic => rng.gen_range(4..=5),
            Rarity::Legendary => MAX_AFFIXES,
        }
    }

    /// Skalierungsfaktor für Affix-Werte.
    fn value_multiplier(self) -> f64 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Magic => 1.2,
            Rarity::Rare => 1.5,
            Rarity::Epic => 1.9,
            Rarity::Legendary => 2.5,
        }
    }
}

// ── Item Slot Type (Equip-Position, hier nur für Anzeige/Filter) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSlot {
    Weapon,
    Offhand,
    Helmet,
    Chest,
    Belt,
    Boots,
    Gloves,
    Ring,
    Amulet,
    Bag,
}

// ── Affix ─────────────────────────────────────────────────────

/// Ein Affix repräsentiert einen einzelnen randomisierten Modifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affix {
    /// Lesbarer Stat-Name, z.B. "Strength", "Crit Chance".
    pub stat: String,
    /// Gerollter Wert.
    pub value: f64,
    /// Angezeigter Suffix/Prefix-Name (z.B. "of the Wolf", "Sturdy").
    pub label: String,
    /// Position im Namen ("prefix" | "suffix").
    pub position: String,
}

/// Affix-Pool-Eintrag für den Random-Roller.
struct AffixDef {
    stat: &'static str,
    label: &'static str,
    /// "prefix" oder "suffix"
    position: &'static str,
    min: f64,
    max: f64,
    /// Wenn true: Wert in Prozent (Anzeige-Hinweis).
    percent: bool,
}

const AFFIX_POOL: &[AffixDef] = &[
    // Prefixes (Min/Max-Werte vor Rarity-Multiplier)
    AffixDef { stat: "Strength",     label: "Mighty",      position: "prefix", min: 1.0, max: 12.0, percent: false },
    AffixDef { stat: "Agility",      label: "Swift",       position: "prefix", min: 1.0, max: 12.0, percent: false },
    AffixDef { stat: "Intellect",    label: "Wise",        position: "prefix", min: 1.0, max: 12.0, percent: false },
    AffixDef { stat: "Stamina",      label: "Sturdy",      position: "prefix", min: 2.0, max: 18.0, percent: false },
    AffixDef { stat: "Armor",        label: "Reinforced",  position: "prefix", min: 3.0, max: 25.0, percent: false },
    AffixDef { stat: "Attack Power", label: "Brutal",      position: "prefix", min: 2.0, max: 20.0, percent: false },
    AffixDef { stat: "Spell Power",  label: "Arcane",      position: "prefix", min: 2.0, max: 20.0, percent: false },
    AffixDef { stat: "Crit Chance",  label: "Vicious",     position: "prefix", min: 1.0, max: 8.0,  percent: true  },
    // Suffixes
    AffixDef { stat: "Gold Find",    label: "of Greed",    position: "suffix", min: 2.0, max: 20.0, percent: true  },
    AffixDef { stat: "Magic Find",   label: "of Fortune",  position: "suffix", min: 1.0, max: 15.0, percent: true  },
    AffixDef { stat: "Life",         label: "of the Bear", position: "suffix", min: 5.0, max: 50.0, percent: false },
    AffixDef { stat: "Mana",         label: "of the Owl",  position: "suffix", min: 5.0, max: 50.0, percent: false },
    AffixDef { stat: "Move Speed",   label: "of Haste",    position: "suffix", min: 1.0, max: 10.0, percent: true  },
    AffixDef { stat: "Strength",     label: "of the Ox",   position: "suffix", min: 1.0, max: 10.0, percent: false },
    AffixDef { stat: "Agility",      label: "of the Wolf", position: "suffix", min: 1.0, max: 10.0, percent: false },
    AffixDef { stat: "Intellect",    label: "of the Sage", position: "suffix", min: 1.0, max: 10.0, percent: false },
];

// ── Item Base Types ───────────────────────────────────────────

struct BaseItem {
    name: &'static str,
    icon: &'static str,
    slot: ItemSlot,
}

const BASE_ITEMS: &[BaseItem] = &[
    BaseItem { name: "Iron Sword",    icon: "\u{1F5E1}\u{FE0F}", slot: ItemSlot::Weapon },
    BaseItem { name: "Steel Dagger",  icon: "\u{1F5E1}\u{FE0F}", slot: ItemSlot::Weapon },
    BaseItem { name: "Hunting Bow",   icon: "\u{1F3F9}",         slot: ItemSlot::Weapon },
    BaseItem { name: "Oak Staff",     icon: "\u{1FA84}",         slot: ItemSlot::Weapon },
    BaseItem { name: "Tower Shield",  icon: "\u{1F6E1}\u{FE0F}", slot: ItemSlot::Offhand },
    BaseItem { name: "Spell Tome",    icon: "\u{1F4D6}",         slot: ItemSlot::Offhand },
    BaseItem { name: "Leather Helm",  icon: "\u{26D1}\u{FE0F}",  slot: ItemSlot::Helmet },
    BaseItem { name: "Plate Armor",   icon: "\u{1F6E1}\u{FE0F}", slot: ItemSlot::Chest },
    BaseItem { name: "Studded Belt",  icon: "\u{1F45A}",         slot: ItemSlot::Belt },
    BaseItem { name: "Cloth Boots",   icon: "\u{1F45F}",         slot: ItemSlot::Boots },
    BaseItem { name: "Iron Gloves",   icon: "\u{1F9E4}",         slot: ItemSlot::Gloves },
    BaseItem { name: "Silver Ring",   icon: "\u{1F48D}",         slot: ItemSlot::Ring },
    BaseItem { name: "Amulet",        icon: "\u{1F4FF}",         slot: ItemSlot::Amulet },
];

// ── Item ──────────────────────────────────────────────────────

/// Eindeutige Item-Instanz mit gerollten Affixen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    /// Voller Anzeigename (Prefix + Base + Suffix), z.B. "Mighty Iron Sword of the Wolf".
    pub name: String,
    /// Nur Base-Name, z.B. "Iron Sword".
    pub base_name: String,
    pub icon: String,
    pub slot: ItemSlot,
    pub rarity: Rarity,
    pub item_level: u32,
    pub affixes: Vec<Affix>,
    /// Gold-Verkaufswert.
    pub vendor_value: f64,
}

/// Roll a brand-new random item.
pub fn roll_random_item(tick: u64, rng: &mut impl Rng) -> Item {
    let base = &BASE_ITEMS[rng.gen_range(0..BASE_ITEMS.len())];
    let rarity = Rarity::roll(rng);
    let item_level = rng.gen_range(1..=50);
    let affix_count = rarity.affix_count(rng);
    let multiplier = rarity.value_multiplier();

    // Affixe ohne Duplikate ziehen.
    let mut chosen: Vec<&AffixDef> = Vec::with_capacity(affix_count);
    let mut attempts = 0;
    while chosen.len() < affix_count && attempts < 50 {
        let candidate = &AFFIX_POOL[rng.gen_range(0..AFFIX_POOL.len())];
        if !chosen.iter().any(|a| std::ptr::eq(*a, candidate as *const _)) {
            chosen.push(candidate);
        }
        attempts += 1;
    }

    let affixes: Vec<Affix> = chosen
        .iter()
        .map(|def| {
            let raw = rng.gen_range(def.min..=def.max) * multiplier;
            // Auf 1 Nachkommastelle runden für saubere Anzeige.
            let value = (raw * 10.0).round() / 10.0;
            let stat_label = if def.percent {
                format!("{} %", def.stat)
            } else {
                def.stat.to_string()
            };
            Affix {
                stat: stat_label,
                value,
                label: def.label.to_string(),
                position: def.position.to_string(),
            }
        })
        .collect();

    let prefix = affixes
        .iter()
        .find(|a| a.position == "prefix")
        .map(|a| format!("{} ", a.label))
        .unwrap_or_default();
    let suffix = affixes
        .iter()
        .find(|a| a.position == "suffix")
        .map(|a| format!(" {}", a.label))
        .unwrap_or_default();

    let full_name = format!("{}{}{}", prefix, base.name, suffix);
    let vendor_value = base_vendor_value(rarity, item_level);

    Item {
        id: format!("item_{}_{}", tick, rng.gen_range(100_000..999_999u32)),
        name: full_name,
        base_name: base.name.to_string(),
        icon: base.icon.to_string(),
        slot: base.slot,
        rarity,
        item_level,
        affixes,
        vendor_value,
    }
}

fn base_vendor_value(rarity: Rarity, ilvl: u32) -> f64 {
    let base = match rarity {
        Rarity::Common => 5.0,
        Rarity::Magic => 25.0,
        Rarity::Rare => 80.0,
        Rarity::Epic => 250.0,
        Rarity::Legendary => 800.0,
    };
    base + (ilvl as f64 * 2.0)
}

// ── Bags ──────────────────────────────────────────────────────

/// Ein einzelner Bag-Container (Backpack, Beutel, Truhe, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bag {
    /// Anzeige-Name.
    pub name: String,
    /// Wenn `true`, kann der Bag nicht entfernt/getauscht werden (Bag 0).
    pub fixed: bool,
    /// Slot-Vektor; `None` = leerer Slot. Länge = Kapazität.
    pub slots: Vec<Option<Item>>,
}

impl Bag {
    pub fn new(name: impl Into<String>, capacity: u32, fixed: bool) -> Self {
        let mut slots = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            slots.push(None);
        }
        Self { name: name.into(), fixed, slots }
    }

    /// Erste freie Slot-Position.
    pub fn first_empty(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }
}

/// Bag-Layout des Spielers: 5 Bag-Slots, Slot 0 = fixed Default Backpack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemBags {
    /// Genau `BAG_SLOT_COUNT` Einträge; `None` = leerer Bag-Slot (kein Container ausgerüstet).
    pub bags: Vec<Option<Bag>>,
}

impl Default for ItemBags {
    fn default() -> Self {
        let mut bags = Vec::with_capacity(BAG_SLOT_COUNT);
        bags.push(Some(Bag::new("Backpack", DEFAULT_BACKPACK_CAPACITY, true)));
        for _ in 1..BAG_SLOT_COUNT {
            bags.push(None);
        }
        Self { bags }
    }
}

impl ItemBags {
    /// Versuche, ein Item in den ersten freien Slot zu legen.
    /// Gibt `Some(item)` zurück, wenn alle Bags voll sind.
    pub fn try_add(&mut self, item: Item) -> Option<Item> {
        for bag_opt in self.bags.iter_mut() {
            if let Some(bag) = bag_opt {
                if let Some(idx) = bag.first_empty() {
                    bag.slots[idx] = Some(item);
                    return None;
                }
            }
        }
        Some(item)
    }

    /// Tausche zwei Slot-Positionen. Liefert `false`, wenn Bag oder Slot ungültig.
    pub fn swap(&mut self, a_bag: usize, a_slot: usize, b_bag: usize, b_slot: usize) -> bool {
        if a_bag >= self.bags.len() || b_bag >= self.bags.len() {
            return false;
        }
        if a_bag == b_bag && a_slot == b_slot {
            return true;
        }

        // Slot-Existenz prüfen.
        let valid = self
            .bags
            .get(a_bag)
            .and_then(|b| b.as_ref())
            .map(|b| a_slot < b.slots.len())
            .unwrap_or(false)
            && self
                .bags
                .get(b_bag)
                .and_then(|b| b.as_ref())
                .map(|b| b_slot < b.slots.len())
                .unwrap_or(false);

        if !valid {
            return false;
        }

        if a_bag == b_bag {
            if let Some(Some(bag)) = self.bags.get_mut(a_bag) {
                bag.slots.swap(a_slot, b_slot);
                return true;
            }
            return false;
        }

        // Cross-bag swap via take/place.
        let item_a = self.bags[a_bag].as_mut().unwrap().slots[a_slot].take();
        let item_b = self.bags[b_bag].as_mut().unwrap().slots[b_slot].take();
        self.bags[a_bag].as_mut().unwrap().slots[a_slot] = item_b;
        self.bags[b_bag].as_mut().unwrap().slots[b_slot] = item_a;
        true
    }

    /// Entferne und liefere das Item an einer Position.
    pub fn take(&mut self, bag: usize, slot: usize) -> Option<Item> {
        self.bags
            .get_mut(bag)
            .and_then(|b| b.as_mut())
            .and_then(|b| b.slots.get_mut(slot))
            .and_then(|s| s.take())
    }

    /// Suche ein Item per ID; liefert (bag, slot).
    pub fn find_position(&self, item_id: &str) -> Option<(usize, usize)> {
        for (bi, bag_opt) in self.bags.iter().enumerate() {
            if let Some(bag) = bag_opt {
                for (si, slot) in bag.slots.iter().enumerate() {
                    if let Some(it) = slot {
                        if it.id == item_id {
                            return Some((bi, si));
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Action Bar ────────────────────────────────────────────────

/// Was kann auf einem Action-Slot liegen?
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActionBinding {
    /// Verweist auf ein Item per ID. Item bleibt im Bag.
    #[serde(rename = "item")]
    Item { item_id: String },
    /// Default-Nahkampfangriff (D2-style left-click).
    #[serde(rename = "attack")]
    Attack,
    /// Spell/Skill — wird beim Drücken via `cast_skill` gewirkt.
    #[serde(rename = "skill")]
    Skill { skill_id: String },
}

/// Action Bar mit `ACTION_BAR_SLOTS` Slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBar {
    /// `None` = leer.
    pub slots: Vec<Option<ActionBinding>>,
}

impl Default for ActionBar {
    fn default() -> Self {
        let mut slots = Vec::with_capacity(ACTION_BAR_SLOTS);
        for _ in 0..ACTION_BAR_SLOTS {
            slots.push(None);
        }
        Self { slots }
    }
}

impl ActionBar {
    pub fn first_empty(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }

    /// Entferne alle Bindings, die auf ein nicht mehr vorhandenes Item zeigen.
    pub fn prune_missing(&mut self, bags: &ItemBags) {
        for slot in self.slots.iter_mut() {
            let drop = match slot {
                Some(ActionBinding::Item { item_id }) => bags.find_position(item_id).is_none(),
                Some(ActionBinding::Attack) => false,
                Some(ActionBinding::Skill { .. }) => false,
                None => false,
            };
            if drop {
                *slot = None;
            }
        }
    }
}

// ── Equipment (D2-style Paperdoll) ───────────────────────────

/// Gear-Slots auf dem Charakter (Paperdoll, wie Diablo 2).
///
/// `Ring` existiert zweimal (`ring1`, `ring2`). Alle anderen Slots
/// nehmen exakt einen `ItemSlot`-Typ auf.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub helmet: Option<Item>,
    pub amulet: Option<Item>,
    pub chest: Option<Item>,
    pub belt: Option<Item>,
    pub gloves: Option<Item>,
    pub boots: Option<Item>,
    pub weapon: Option<Item>,
    pub offhand: Option<Item>,
    pub ring1: Option<Item>,
    pub ring2: Option<Item>,
}

/// Welcher konkrete Equipment-Slot soll bedient werden?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlotName {
    Helmet,
    Amulet,
    Chest,
    Belt,
    Gloves,
    Boots,
    Weapon,
    Offhand,
    Ring1,
    Ring2,
}

impl Equipment {
    /// Mutable Zugriff auf den passenden Slot.
    fn slot_mut(&mut self, name: EquipSlotName) -> &mut Option<Item> {
        match name {
            EquipSlotName::Helmet => &mut self.helmet,
            EquipSlotName::Amulet => &mut self.amulet,
            EquipSlotName::Chest => &mut self.chest,
            EquipSlotName::Belt => &mut self.belt,
            EquipSlotName::Gloves => &mut self.gloves,
            EquipSlotName::Boots => &mut self.boots,
            EquipSlotName::Weapon => &mut self.weapon,
            EquipSlotName::Offhand => &mut self.offhand,
            EquipSlotName::Ring1 => &mut self.ring1,
            EquipSlotName::Ring2 => &mut self.ring2,
        }
    }

    /// Akzeptiert das Slot-Name den Item-Typ?
    pub fn accepts(name: EquipSlotName, item_slot: ItemSlot) -> bool {
        match (name, item_slot) {
            (EquipSlotName::Helmet, ItemSlot::Helmet) => true,
            (EquipSlotName::Amulet, ItemSlot::Amulet) => true,
            (EquipSlotName::Chest, ItemSlot::Chest) => true,
            (EquipSlotName::Belt, ItemSlot::Belt) => true,
            (EquipSlotName::Gloves, ItemSlot::Gloves) => true,
            (EquipSlotName::Boots, ItemSlot::Boots) => true,
            (EquipSlotName::Weapon, ItemSlot::Weapon) => true,
            (EquipSlotName::Offhand, ItemSlot::Offhand) => true,
            (EquipSlotName::Ring1 | EquipSlotName::Ring2, ItemSlot::Ring) => true,
            _ => false,
        }
    }

    /// Default-Slot für einen Item-Typ. Bei Ringen wird zuerst Ring1 genutzt,
    /// wenn frei, sonst Ring2 (Auto-Equip-Komfort).
    pub fn default_slot_for(&self, item_slot: ItemSlot) -> Option<EquipSlotName> {
        match item_slot {
            ItemSlot::Helmet => Some(EquipSlotName::Helmet),
            ItemSlot::Amulet => Some(EquipSlotName::Amulet),
            ItemSlot::Chest => Some(EquipSlotName::Chest),
            ItemSlot::Belt => Some(EquipSlotName::Belt),
            ItemSlot::Gloves => Some(EquipSlotName::Gloves),
            ItemSlot::Boots => Some(EquipSlotName::Boots),
            ItemSlot::Weapon => Some(EquipSlotName::Weapon),
            ItemSlot::Offhand => Some(EquipSlotName::Offhand),
            ItemSlot::Ring => {
                if self.ring1.is_none() {
                    Some(EquipSlotName::Ring1)
                } else {
                    Some(EquipSlotName::Ring2)
                }
            }
            ItemSlot::Bag => None,
        }
    }

    /// Setzt ein Item in den Slot. Liefert das vorher dort liegende Item zurück
    /// (oder `None`, wenn Slot leer war oder Typ nicht passt — dann wird auch
    /// nichts gesetzt und das übergebene Item zurückgegeben in `Err`).
    pub fn equip(&mut self, name: EquipSlotName, item: Item) -> Result<Option<Item>, Item> {
        if !Self::accepts(name, item.slot) {
            return Err(item);
        }
        let prev = self.slot_mut(name).take();
        *self.slot_mut(name) = Some(item);
        Ok(prev)
    }

    /// Nimmt das Item aus dem Slot.
    pub fn unequip(&mut self, name: EquipSlotName) -> Option<Item> {
        self.slot_mut(name).take()
    }

    /// Aggregierte Affix-Werte über alle Slots (für Stat-Display).
    pub fn aggregate_stats(&self) -> std::collections::HashMap<String, f64> {
        use std::collections::HashMap;
        let mut totals: HashMap<String, f64> = HashMap::new();
        let slots = [
            &self.helmet,
            &self.amulet,
            &self.chest,
            &self.belt,
            &self.gloves,
            &self.boots,
            &self.weapon,
            &self.offhand,
            &self.ring1,
            &self.ring2,
        ];
        for slot in slots.iter().flat_map(|s| s.iter()) {
            for affix in &slot.affixes {
                *totals.entry(affix.stat.clone()).or_insert(0.0) += affix.value;
            }
        }
        totals
    }
}
