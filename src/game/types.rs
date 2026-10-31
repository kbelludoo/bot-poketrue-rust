use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    pub credentials: GameCredentials,
    #[serde(default)]
    pub settings: GameSettings,
    #[serde(rename = "bindIP", default)]
    pub bind_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCredentials {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameSettings {
    #[serde(rename = "huntTarget", alias = "hunt_target", default)]
    pub hunt_target: String,
    #[serde(rename = "autoHunt", alias = "auto_hunt", default = "default_true")]
    pub auto_hunt: bool,
    #[serde(rename = "autoSell", alias = "auto_sell", default = "default_true")]
    pub auto_sell: bool,
    #[serde(rename = "autoHeal", alias = "auto_heal", default = "default_true")]
    pub auto_heal: bool,
    #[serde(rename = "autoDaily", alias = "auto_daily", default = "default_true")]
    pub auto_daily: bool,
    #[serde(rename = "autoCatch", alias = "auto_catch", default = "default_true")]
    pub auto_catch: bool,
    #[serde(rename = "autoTreino", alias = "auto_treino", default = "default_true")]
    pub auto_treino: bool,
    #[serde(rename = "autoEquip", alias = "auto_equip", default = "default_true")]
    pub auto_equip: bool,
    #[serde(rename = "autoEvo", alias = "auto_evo", default = "default_true")]
    pub auto_evo: bool,
    #[serde(rename = "autoBoss", alias = "auto_boss", default = "default_true")]
    pub auto_boss: bool,
    #[serde(rename = "focusBestiary", alias = "focus_bestiary", default = "default_true")]
    pub focus_bestiary: bool,
    #[serde(rename = "ballType", alias = "ball_type", default)]
    pub ball_type: Option<String>,
    #[serde(rename = "healThreshold", alias = "heal_threshold", default)]
    pub heal_threshold: Option<f64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    #[serde(default)]
    pub uid: String,
    #[serde(default, alias = "id", alias = "poke_id", alias = "species_id")]
    pub species_id: u32,
    #[serde(default, alias = "nome")]
    pub name: String,
    #[serde(default, alias = "lvl")]
    pub level: u32,
    #[serde(default)]
    pub hp: u32,
    #[serde(default, alias = "maxHp", alias = "totalHp")]
    pub max_hp: u32,
    #[serde(default, alias = "heldItem", alias = "held_item", alias = "item")]
    pub held_item: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PokeTrueState {
    #[serde(default, alias = "nome", alias = "name")]
    pub player_name: String,
    #[serde(default, alias = "lvl")]
    pub level: u32,
    #[serde(default, alias = "xp")]
    pub exp: u64,
    #[serde(default)]
    pub gold: u64,
    #[serde(default, alias = "time", alias = "pokemons")]
    pub team: Vec<Pokemon>,
    #[serde(default, alias = "hunt", alias = "current_hunt")]
    pub current_hunt: Option<String>,
    #[serde(default)]
    pub raw_estado: Option<Value>,
}
