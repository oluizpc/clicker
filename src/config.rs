//! Configuração persistida: settings atuais + perfis nomeados, salvos em JSON.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub min_cps: f32,
    pub max_cps: f32,
    pub jitter: bool,
    /// 0 = esquerdo, 1 = direito, 2 = meio
    pub button: u8,
    /// 0 = toggle (liga/desliga), 1 = hold (segurar hotkey), 2 = segurar botão esquerdo do mouse
    pub mode: u8,
    pub hotkey_vk: i32,
    pub hotkey_name: String,
    pub start_delay_ms: u64,
    pub burst_clicks: u32,
    pub click_limit_enabled: bool,
    pub click_limit: u64,
    pub target_window_only: bool,
    pub target_window_title: String,
    pub dark_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            min_cps: 12.0,
            max_cps: 16.0,
            jitter: true,
            button: 0,
            mode: 0,
            hotkey_vk: 0x75, // F6
            hotkey_name: "F6".into(),
            start_delay_ms: 0,
            burst_clicks: 25,
            click_limit_enabled: false,
            click_limit: 1000,
            target_window_only: false,
            target_window_title: "Roblox".into(),
            dark_mode: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub active: Settings,
    pub profiles: BTreeMap<String, Settings>,
}

fn config_path() -> PathBuf {
    // Salva ao lado do executável; cai para o diretório atual se necessário.
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.push("autoclicker_config.json");
    dir
}

impl Config {
    pub fn load() -> Self {
        match std::fs::read_to_string(config_path()) {
            Ok(txt) => serde_json::from_str(&txt).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) {
        if let Ok(txt) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), txt);
        }
    }
}
