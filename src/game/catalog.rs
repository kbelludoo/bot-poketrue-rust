use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const HUNTS_RAW: &str = include_str!("data/hunts.json");
pub const EVOLUCOES_RAW: &str = include_str!("data/evolucoes.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaCandidate {
    pub id: String,
    pub name: String,
    pub min_level: u32,
    pub max_level: u32,
    pub base_exp: f64,
    pub base_gold: f64,
}

#[derive(Clone, Default)]
pub struct Catalog;

static PARSED_HUNTS: OnceLock<Vec<AreaCandidate>> = OnceLock::new();

impl Catalog {
    pub fn get_all_hunts() -> &'static [AreaCandidate] {
        PARSED_HUNTS.get_or_init(|| {
            let mut list = Vec::new();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(HUNTS_RAW) {
                if let Some(arr) = parsed.as_array() {
                    for item in arr {
                        let id = item.get("slug")
                            .or_else(|| item.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let name = item.get("nome")
                            .or_else(|| item.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&id)
                            .to_string();
                        let (min_level, max_level) = if let Some(nivel) = item.get("nivel").and_then(|v| v.as_array()) {
                            let min = nivel.get(0).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                            let max = nivel.get(1).and_then(|v| v.as_u64()).unwrap_or(100) as u32;
                            (min, max)
                        } else {
                            let min = item.get("minLevel").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                            let max = item.get("maxLevel").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
                            (min, max)
                        };
                        let base_exp = item.get("killsPorHora").and_then(|v| v.as_f64()).unwrap_or(100.0);
                        let base_gold = item.get("baseGold").and_then(|v| v.as_f64()).unwrap_or(50.0);

                        list.push(AreaCandidate {
                            id,
                            name,
                            min_level,
                            max_level,
                            base_exp,
                            base_gold,
                        });
                    }
                }
            }
            if list.is_empty() {
                list.push(AreaCandidate {
                    id: "floresta_de_viridian".to_string(),
                    name: "Floresta de Viridian".to_string(),
                    min_level: 1,
                    max_level: 30,
                    base_exp: 250.0,
                    base_gold: 120.0,
                });
            }
            list
        })
    }

    pub fn select_best_hunt(&self, level: u32) -> &'static AreaCandidate {
        let all = Self::get_all_hunts();
        for area in all {
            if level >= area.min_level && level <= area.max_level {
                return area;
            }
        }
        &all[0]
    }
}
