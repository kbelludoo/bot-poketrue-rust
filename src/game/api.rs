use crate::game::types::{AccountConfig, PokeTrueState, Pokemon};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

pub const DEFAULT_BASE_URL: &str = "https://poketrue.net/api/";
pub const USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

pub struct PokeTrueClient {
    client: Client,
    base_url: String,
    token: String,
    window_id: String,
}

impl PokeTrueClient {
    pub fn new(token: &str, origin: Option<&str>) -> Self {
        let base = origin.unwrap_or(DEFAULT_BASE_URL).trim_end_matches('/').to_string() + "/";
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let window_id = format!("{:x}{:x}", rand::random::<u64>(), now_ms);

        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base,
            token: token.to_string(),
            window_id,
        }
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        headers.insert("accept", HeaderValue::from_static("application/json"));
        headers.insert("sec-ch-ua", HeaderValue::from_static(
            r#""Chromium";v="128", "Not;A=Brand";v="24", "Google Chrome";v="128""#
        ));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static(r#""Windows""#));
        headers.insert("x-janela", HeaderValue::from_str(&self.window_id).unwrap());

        if !self.token.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", self.token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    pub async fn get_state(&self) -> Result<PokeTrueState, String> {
        let url = format!("{}state", self.base_url);
        let resp = self.client.get(&url)
            .headers(self.default_headers())
            .send()
            .await
            .map_err(|e| format!("Falha de conexão em GET state: {e}"))?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| format!("Falha ao ler state: {e}"))?;

        if !status.is_success() {
            return Err(format!("GET state falhou com HTTP {}: {}", status, body));
        }

        let parsed: Value = serde_json::from_str(&body).map_err(|e| format!("Erro no JSON do state: {e}"))?;
        let estado = parsed.get("estado").unwrap_or(&parsed);

        let player_name = estado.get("nome").and_then(|v| v.as_str()).unwrap_or("Treinador").to_string();
        let level = estado.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let exp = estado.get("exp").and_then(|v| v.as_u64()).unwrap_or(0);
        let gold = estado.get("gold").and_then(|v| v.as_u64()).unwrap_or(0);
        let current_hunt = estado.get("currentHunt").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut team = Vec::new();
        if let Some(team_arr) = estado.get("time").and_then(|v| v.as_array()) {
            for poke in team_arr {
                team.push(Pokemon {
                    uid: poke.get("uid").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    species_id: poke.get("id").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    name: poke.get("name").and_then(|v| v.as_str()).unwrap_or("Pokemon").to_string(),
                    level: poke.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    hp: poke.get("hp").and_then(|v| v.as_u64()).unwrap_or(100) as u32,
                    max_hp: poke.get("maxHp").and_then(|v| v.as_u64()).unwrap_or(100) as u32,
                    held_item: poke.get("heldItem").and_then(|v| v.as_str()).map(|s| s.to_string()),
                });
            }
        }

        Ok(PokeTrueState {
            player_name,
            level,
            exp,
            gold,
            team,
            current_hunt,
            raw_estado: Some(parsed),
        })
    }

    pub async fn start_hunt(&self, hunt_id: &str) -> Result<Value, String> {
        let url = format!("{}hunt/start", self.base_url);
        let mut headers = self.default_headers();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({ "huntId": hunt_id });
        let resp = self.client.post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("hunt/start network: {e}"))?;

        resp.json().await.map_err(|e| format!("hunt/start json: {e}"))
    }

    pub async fn stop_hunt(&self) -> Result<Value, String> {
        let url = format!("{}hunt/stop", self.base_url);
        let resp = self.client.post(&url)
            .headers(self.default_headers())
            .send()
            .await
            .map_err(|e| format!("hunt/stop network: {e}"))?;

        resp.json().await.map_err(|e| format!("hunt/stop json: {e}"))
    }

    pub async fn heal_jenny(&self) -> Result<Value, String> {
        let url = format!("{}jenny/heal", self.base_url);
        let resp = self.client.post(&url)
            .headers(self.default_headers())
            .send()
            .await
            .map_err(|e| format!("jenny/heal: {e}"))?;

        resp.json().await.map_err(|e| format!("jenny/heal json: {e}"))
    }

    pub async fn claim_treino(&self) -> Result<Value, String> {
        let url = format!("{}treino/claim", self.base_url);
        let resp = self.client.post(&url)
            .headers(self.default_headers())
            .send()
            .await
            .map_err(|e| format!("treino/claim: {e}"))?;

        resp.json().await.map_err(|e| format!("treino/claim json: {e}"))
    }
}
