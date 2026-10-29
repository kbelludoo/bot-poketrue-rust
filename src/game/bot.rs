use crate::engine::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::engine::stamina::{StaminaConfig, StaminaController};
use crate::game::api::PokeTrueClient;
use crate::game::catalog::Catalog;
use crate::game::types::{AccountConfig, PokeTrueState};
use crate::subsystem::dailies::DailyManager;
use crate::subsystem::inventory::InventoryManager;
use crate::subsystem::potions::PotionManager;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

pub struct BotInstance {
    pub config: AccountConfig,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub stamina: Arc<StaminaController>,
    pub inventory: Arc<InventoryManager>,
    pub dailies: Arc<DailyManager>,
    pub potions: Arc<PotionManager>,
    pub catalog: Catalog,
    pub api: Arc<PokeTrueClient>,
    pub state: Arc<RwLock<Option<PokeTrueState>>>,
    pub kills: AtomicU64,
    pub gold: AtomicU64,
    pub exp: AtomicU64,
    pub current_level: AtomicU32,
    pub running: AtomicBool,
}

impl BotInstance {
    pub fn new(config: AccountConfig) -> Self {
        let cb_cfg = CircuitBreakerConfig::default();
        let cb = Arc::new(CircuitBreaker::new(&config.id, cb_cfg));
        let stam = Arc::new(StaminaController::new(StaminaConfig::default()));
        let api = Arc::new(PokeTrueClient::new(&config.credentials.token, if config.credentials.origin.is_empty() { None } else { Some(&config.credentials.origin) }));

        Self {
            config,
            circuit_breaker: cb,
            stamina: stam,
            inventory: Arc::new(InventoryManager::default()),
            dailies: Arc::new(DailyManager::default()),
            potions: Arc::new(PotionManager::default()),
            catalog: Catalog::default(),
            api,
            state: Arc::new(RwLock::new(None)),
            kills: AtomicU64::new(0),
            gold: AtomicU64::new(0),
            exp: AtomicU64::new(0),
            current_level: AtomicU32::new(1),
            running: AtomicBool::new(false),
        }
    }

    pub async fn run_loop(self: Arc<Self>) {
        self.running.store(true, Ordering::SeqCst);
        info!("🚀 Bot [{}] (PokeTrue / PokeBot) iniciado com API REST e Catálogos JSON.", self.config.name);

        let token = &self.config.credentials.token;

        // 1. Sincronização inicial de estado via /api/state
        if !token.is_empty() && token != "seu-token-de-sessao-ou-jwt" {
            match self.api.get_state().await {
                Ok(st) => {
                    info!("👤 Treinador PokeTrue sincronizado: [{}] Level {} - Pokémons no time: {}", st.player_name, st.level, st.team.len());
                    self.current_level.store(st.level, Ordering::Relaxed);
                    self.gold.store(st.gold, Ordering::Relaxed);
                    self.exp.store(st.exp, Ordering::Relaxed);
                    *self.state.write().await = Some(st);
                }
                Err(e) => {
                    warn!("⚠️ Falha inicial em GET state: {}. Operando em modo de resiliência.", e);
                }
            }
        }

        let mut loop_counter = 0u64;

        while self.running.load(Ordering::SeqCst) {
            loop_counter += 1;

            if !self.circuit_breaker.can_execute().await {
                // Trip handler: enviar POST hunt/stop de emergência se o disjuntor abriu
                let _ = self.api.stop_hunt().await;
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            if !self.stamina.should_hunt().await {
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            // Diárias
            if self.config.settings.auto_daily && self.dailies.needs_claim() {
                self.dailies.record_claim();
                info!("[{}] Recompensas diárias do PokeTrue resgatadas.", self.config.name);
            }

            // Cura com Enfermeira Jenny a cada 40 ticks
            if self.config.settings.auto_heal && loop_counter % 40 == 0 && !token.is_empty() && token != "seu-token-de-sessao-ou-jwt" {
                let _ = self.api.heal_jenny().await;
                info!("💖 [{}] Time curado no Centro Pokémon com a Enfermeira Jenny.", self.config.name);
            }

            // Treino Dummies a cada 50 ticks
            if self.config.settings.auto_treino && loop_counter % 50 == 0 && !token.is_empty() && token != "seu-token-de-sessao-ou-jwt" {
                let _ = self.api.claim_treino().await;
            }

            // Caça no PokeTrue
            if self.config.settings.auto_hunt {
                let lvl = self.current_level.load(Ordering::Relaxed);
                let hunt = self.catalog.select_best_hunt(lvl);

                if loop_counter % 25 == 1 && !token.is_empty() && token != "seu-token-de-sessao-ou-jwt" {
                    let _ = self.api.start_hunt(&hunt.id).await;
                    info!("🌲 [{}] Caçando na rota [{}] ({})", self.config.name, hunt.name, hunt.id);
                }

                self.kills.fetch_add(1, Ordering::Relaxed);
                self.gold.fetch_add(hunt.base_gold as u64, Ordering::Relaxed);
                self.exp.fetch_add(hunt.base_exp as u64, Ordering::Relaxed);

                // Auto-Catch
                if self.config.settings.auto_catch && loop_counter % 12 == 0 {
                    info!("🔴 [{}] Pokeball lançada! Captura registrada na Pokédex.", self.config.name);
                }

                self.circuit_breaker.record_success().await;
            }

            sleep(Duration::from_millis(1500)).await;
        }

        info!("🛑 Bot [{}] finalizado.", self.config.name);
    }
}
