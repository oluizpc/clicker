//! Estado compartilhado e as threads de fundo: hotkey e clique.

use crate::config::Settings;
use crate::input::{self, MouseButton};
use rand::Rng;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Shared {
    pub settings: Mutex<Settings>,
    /// true enquanto o clique automático está ativo.
    pub clicking: AtomicBool,
    /// mantém as threads vivas; vira false ao fechar.
    pub running: AtomicBool,
    /// pedido de rajada (burst) de uma vez só.
    pub burst_request: AtomicBool,
    /// cliques já enviados na ativação atual.
    pub clicks_done: AtomicU64,
    /// CPS alvo calculado no último clique (para exibir).
    pub cps_now: Mutex<f32>,
    /// sinaliza que a ativação acabou de começar (resetar contador/delay).
    reset: AtomicBool,
}

impl Shared {
    pub fn new(settings: Settings) -> Self {
        Shared {
            settings: Mutex::new(settings),
            clicking: AtomicBool::new(false),
            running: AtomicBool::new(true),
            burst_request: AtomicBool::new(false),
            clicks_done: AtomicU64::new(0),
            cps_now: Mutex::new(0.0),
            reset: AtomicBool::new(false),
        }
    }

    fn set_clicking(&self, on: bool) {
        let was = self.clicking.swap(on, Ordering::SeqCst);
        if on && !was {
            self.reset.store(true, Ordering::SeqCst);
        }
    }

    /// Liga/desliga pelo botão da interface (usado só no modo toggle).
    pub fn ui_toggle(&self) {
        let now = self.clicking.load(Ordering::SeqCst);
        self.set_clicking(!now);
    }
}

/// Thread que lê a hotkey e liga/desliga o clique conforme o modo.
pub fn spawn_hotkey(shared: std::sync::Arc<Shared>) {
    std::thread::spawn(move || {
        let mut prev_down = false;
        while shared.running.load(Ordering::SeqCst) {
            let (vk, mode) = {
                let s = shared.settings.lock().unwrap();
                (s.hotkey_vk, s.mode)
            };
            match mode {
                1 => {
                    // Hold: ativo enquanto a hotkey estiver pressionada.
                    let down = input::key_down(vk);
                    if down != shared.clicking.load(Ordering::SeqCst) {
                        shared.set_clicking(down);
                    }
                    prev_down = down;
                }
                2 => {
                    // Segurar botão esquerdo do mouse (VK_LBUTTON = 0x01).
                    let down = input::key_down(0x01);
                    if down != shared.clicking.load(Ordering::SeqCst) {
                        shared.set_clicking(down);
                    }
                    prev_down = down;
                }
                _ => {
                    // Toggle: inverte na borda de subida (tecla recém-pressionada).
                    let down = input::key_down(vk);
                    if down && !prev_down {
                        let now = shared.clicking.load(Ordering::SeqCst);
                        shared.set_clicking(!now);
                    }
                    prev_down = down;
                }
            }
            // Polling curto para o modo "segurar mouse" responder rápido.
            std::thread::sleep(Duration::from_millis(2));
        }
    });
}

/// Thread que efetivamente envia os cliques.
pub fn spawn_clicker(shared: std::sync::Arc<Shared>) {
    std::thread::spawn(move || {
        let mut rng = rand::thread_rng();
        while shared.running.load(Ordering::SeqCst) {
            // Rajada avulsa (botão "Disparar rajada"), independente do toggle.
            if shared.burst_request.swap(false, Ordering::SeqCst) {
                let (btn, n, cps) = {
                    let s = shared.settings.lock().unwrap();
                    (MouseButton::from_u8(s.button), s.burst_clicks, s.max_cps.max(1.0))
                };
                let gap = Duration::from_secs_f32(1.0 / cps);
                for _ in 0..n {
                    input::send_click(btn);
                    std::thread::sleep(gap);
                }
            }

            if !shared.clicking.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }

            // Início de uma ativação: aplica delay e zera o contador.
            if shared.reset.swap(false, Ordering::SeqCst) {
                shared.clicks_done.store(0, Ordering::SeqCst);
                let delay = shared.settings.lock().unwrap().start_delay_ms;
                if delay > 0 {
                    sleep_interruptible(&shared, Duration::from_millis(delay));
                }
            }

            let s = shared.settings.lock().unwrap().clone();

            // Filtro de janela-alvo.
            if s.target_window_only && !s.target_window_title.trim().is_empty() {
                let title = input::foreground_title().to_lowercase();
                if !title.contains(&s.target_window_title.to_lowercase()) {
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
            }

            // Limite de cliques.
            if s.click_limit_enabled && shared.clicks_done.load(Ordering::SeqCst) >= s.click_limit {
                shared.set_clicking(false);
                continue;
            }

            // CPS alvo (com jitter opcional).
            let cps = if s.jitter && s.max_cps > s.min_cps {
                rng.gen_range(s.min_cps..=s.max_cps)
            } else {
                s.min_cps
            }
            .max(0.5);

            input::send_click(MouseButton::from_u8(s.button));
            shared.clicks_done.fetch_add(1, Ordering::SeqCst);
            *shared.cps_now.lock().unwrap() = cps;

            sleep_interruptible(&shared, Duration::from_secs_f32(1.0 / cps));
        }
    });
}

/// Espera de alta precisão: dorme a maior parte e faz spin nos últimos ~1,5 ms
/// para o CPS real bater com o configurado, mesmo em taxas altas. Acorda cedo
/// se o clique for desligado (resposta instantânea).
fn sleep_interruptible(shared: &Shared, dur: Duration) {
    let end = Instant::now() + dur;
    loop {
        if !shared.clicking.load(Ordering::SeqCst) || !shared.running.load(Ordering::SeqCst) {
            break;
        }
        let now = Instant::now();
        if now >= end {
            break;
        }
        let remaining = end - now;
        if remaining > Duration::from_micros(1500) {
            // Ainda falta bastante: dorme fino para não gastar CPU à toa.
            std::thread::sleep(Duration::from_micros(500));
        } else {
            // Reta final: spin ocupado para precisão de sub-milissegundo.
            std::hint::spin_loop();
        }
    }
}
