//! Interface gráfica (egui) — sidebar de navegação + páginas.
//! Ao abrir, uma tela inicial pede para escolher a janela onde o clicker atua.

use crate::config::{Config, Settings};
use crate::theme::{self, Palette};
use crate::worker::Shared;
use egui::RichText;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Título da própria janela do app, para removê-la da lista de alvos.
const SELF_TITLE: &str = "DzkLP - Clicker";

/// Verde de "ativo" usado na pílula de status e botões.
const GREEN: egui::Color32 = egui::Color32::from_rgb(0x3F, 0xC5, 0x6B);

/// Telas do app: primeiro escolher a janela, depois a interface principal.
#[derive(PartialEq)]
enum Screen {
    Picker,
    Main,
}

/// Páginas do menu lateral. Adicione novas variantes aqui para novos módulos.
#[derive(PartialEq, Clone, Copy)]
enum Page {
    Clicker,
    Profiles,
    Appearance,
}

/// Lista as janelas abertas, sem a própria janela do app nem títulos vazios.
fn window_choices() -> Vec<String> {
    crate::input::list_windows()
        .into_iter()
        .filter(|t| !t.trim().is_empty() && t != SELF_TITLE)
        .collect()
}

/// Palavras que indicam uma janela de Minecraft (jogo, clientes ou launchers).
const MINECRAFT_HINTS: &[&str] = &[
    "minecraft",
    "lunar",
    "badlion",
    "feather",
    "labymod",
    "salwyrr",
    "pvp lounge",
    "pvplounge",
    "forge",
    "fabric",
    "optifine",
    "rlcraft",
    "tlauncher",
    "multimc",
    "prism",
    "atlauncher",
    "curseforge",
    "modrinth",
    "technic",
    "ftb",
    "polymc",
    "legacylauncher",
];

/// Heurística simples: o título parece ser de algo ligado ao Minecraft?
fn minecraft_like(title: &str) -> bool {
    let t = title.to_lowercase();
    MINECRAFT_HINTS.iter().any(|h| t.contains(h))
}

/// Encurta um texto longo acrescentando "…" no fim.
fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    }
}

/// Rótulo e cor do "selo" do cliente, a partir do título da janela.
fn client_badge(title: &str) -> (&'static str, egui::Color32) {
    use egui::Color32 as C;
    let t = title.to_lowercase();
    let has = |k: &str| t.contains(k);
    if has("lunar") {
        ("Lunar", C::from_rgb(0x5B, 0xA8, 0xFF))
    } else if has("badlion") {
        ("Badlion", C::from_rgb(0xE0, 0x5A, 0x2B))
    } else if has("feather") {
        ("Feather", C::from_rgb(0x8E, 0xD1, 0xFF))
    } else if has("labymod") {
        ("LabyMod", C::from_rgb(0xB1, 0x7A, 0xF0))
    } else if has("salwyrr") {
        ("Salwyrr", C::from_rgb(0xF0, 0x7A, 0xC0))
    } else if has("pvp lounge") || has("pvplounge") {
        ("PvP Lounge", GREEN)
    } else if has("optifine") {
        ("OptiFine", C::from_rgb(0x7F, 0xB0, 0x5A))
    } else if has("forge") {
        ("Forge", C::from_rgb(0x8A, 0x9A, 0xB0))
    } else if has("fabric") {
        ("Fabric", C::from_rgb(0xC9, 0xB4, 0x8F))
    } else if has("tlauncher")
        || has("multimc")
        || has("polymc")
        || has("prism")
        || has("atlauncher")
        || has("curseforge")
        || has("modrinth")
        || has("technic")
        || has("ftb")
        || has("legacylauncher")
    {
        ("Launcher", C::from_rgb(0x9A, 0xA0, 0xB0))
    } else {
        ("Minecraft", C::from_rgb(0x5F, 0xB8, 0x5F))
    }
}

/// Desenha uma linha de janela estilo "card": selo colorido, título e
/// rótulo do cliente à direita. Retorna a resposta para tratar clique.
fn window_row(ui: &mut egui::Ui, p: &Palette, title: &str, selected: bool) -> egui::Response {
    let (label, dot) = client_badge(title);
    let w = ui.available_width();
    let h = 30.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let painter = ui.painter();

    let bg = if selected {
        p.surface2
    } else if resp.hovered() {
        p.surface2.linear_multiply(0.55)
    } else {
        p.surface
    };
    painter.rect_filled(rect, egui::Rounding::same(7.0), bg);
    if selected {
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, h));
        painter.rect_filled(bar, egui::Rounding::same(2.0), p.accent);
    }

    let cy = rect.center().y;
    let dot_x = rect.min.x + 16.0;
    painter.circle_filled(egui::pos2(dot_x, cy), 4.0, dot);

    let short: String = title.chars().take(60).collect();
    painter.text(
        egui::pos2(dot_x + 12.0, cy),
        egui::Align2::LEFT_CENTER,
        short,
        egui::FontId::proportional(13.5),
        p.text,
    );
    painter.text(
        egui::pos2(rect.max.x - 10.0, cy),
        egui::Align2::RIGHT_CENTER,
        label,
        egui::FontId::proportional(11.5),
        p.muted,
    );
    resp
}

/// Item do menu lateral: faixa clicável de largura total, acento quando ativo.
fn nav_item(ui: &mut egui::Ui, p: &Palette, label: &str, selected: bool) -> bool {
    let w = ui.available_width();
    let h = 36.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let painter = ui.painter();

    let bg = if selected {
        p.accent.linear_multiply(0.16)
    } else if resp.hovered() {
        p.surface2
    } else {
        egui::Color32::TRANSPARENT
    };
    painter.rect_filled(rect, egui::Rounding::same(8.0), bg);
    if selected {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + 7.0),
            egui::vec2(3.0, h - 14.0),
        );
        painter.rect_filled(bar, egui::Rounding::same(2.0), p.accent);
    }
    let col = if selected { p.accent } else { p.text };
    painter.text(
        egui::pos2(rect.min.x + 14.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.5),
        col,
    );
    resp.clicked()
}

/// Nome legível para um virtual-key code do Windows.
fn vk_name(vk: i32) -> String {
    match vk {
        0x01 => "Mouse Esq".into(),
        0x02 => "Mouse Dir".into(),
        0x04 => "Mouse Meio".into(),
        0x05 => "Mouse 4".into(),
        0x06 => "Mouse 5".into(),
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x10 => "Shift".into(),
        0x11 => "Ctrl".into(),
        0x12 => "Alt".into(),
        0x14 => "CapsLock".into(),
        0x1B => "Esc".into(),
        0x20 => "Espaço".into(),
        0x21 => "PageUp".into(),
        0x22 => "PageDown".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "Seta Esq".into(),
        0x26 => "Seta Cima".into(),
        0x27 => "Seta Dir".into(),
        0x28 => "Seta Baixo".into(),
        0x2D => "Insert".into(),
        0x2E => "Delete".into(),
        0x30..=0x39 => ((vk as u8) as char).to_string(),
        0x41..=0x5A => ((vk as u8) as char).to_string(),
        0x60..=0x69 => format!("Num {}", vk - 0x60),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0xA0 => "Shift Esq".into(),
        0xA1 => "Shift Dir".into(),
        0xA2 => "Ctrl Esq".into(),
        0xA3 => "Ctrl Dir".into(),
        0xA4 => "Alt Esq".into(),
        0xA5 => "Alt Dir".into(),
        _ => format!("Tecla 0x{vk:02X}"),
    }
}

pub struct App {
    shared: Arc<Shared>,
    config: Config,
    pal: Palette,
    new_profile_name: String,
    status: String,
    capturing: bool,
    page: Page,
    // Tela inicial de seleção de janela.
    screen: Screen,
    win_list: Vec<String>,
    win_filter: String,
    picked: Option<String>,
    mc_only: bool,
    // Medição de CPS real (cliques por segundo observados).
    meas_cps: f32,
    meas_t: Instant,
    meas_clicks: u64,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, shared: Arc<Shared>, config: Config) -> Self {
        let pal = theme::palette(config.active.dark_mode);
        theme::install(&cc.egui_ctx, &pal);

        let win_list = window_choices();
        let saved = config.active.target_window_title.trim().to_string();
        let picked = win_list.iter().find(|t| **t == saved).cloned();

        App {
            shared,
            config,
            pal,
            new_profile_name: String::new(),
            status: String::new(),
            capturing: false,
            page: Page::Clicker,
            screen: Screen::Picker,
            win_list,
            win_filter: String::new(),
            picked,
            mc_only: true,
            meas_cps: 0.0,
            meas_t: Instant::now(),
            meas_clicks: 0,
        }
    }

    fn persist(&self) {
        let mut cfg = self.config.clone();
        cfg.active = self.shared.settings.lock().unwrap().clone();
        cfg.save();
    }

    /// Aplica a escolha da tela inicial e entra no painel.
    fn confirm_window(&mut self, scoped: bool) {
        let mut s = self.shared.settings.lock().unwrap().clone();
        s.target_window_only = scoped;
        if scoped {
            if let Some(t) = &self.picked {
                s.target_window_title = t.clone();
            }
        }
        let name = s.target_window_title.clone();
        *self.shared.settings.lock().unwrap() = s;
        self.persist();
        self.status = if scoped {
            format!("Clicker vai atuar em: {name}")
        } else {
            "Clicker vai atuar em qualquer janela.".to_string()
        };
        self.screen = Screen::Main;
    }

    /// Volta para a tela de seleção e atualiza a lista de janelas.
    fn open_picker(&mut self) {
        self.win_list = window_choices();
        let current = self.shared.settings.lock().unwrap().target_window_title.clone();
        self.picked = self.win_list.iter().find(|t| **t == current).cloned();
        self.screen = Screen::Picker;
    }

    /// Tela inicial: escolher a janela onde o clicker vai funcionar.
    fn pick_window(&mut self, ctx: &egui::Context) {
        let p = self.pal;
        ctx.request_repaint_after(Duration::from_millis(200));

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_width();
            let col_w = avail.min(560.0);
            let pad = ((avail - col_w) / 2.0).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(pad);
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.add_space(28.0);

                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, egui::Rounding::same(10.0), p.accent);
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "D",
                            egui::FontId::new(22.0, egui::FontFamily::Proportional),
                            p.on_accent,
                        );
                        ui.add_space(10.0);
                        ui.label(RichText::new("DzkLP Clicker").size(20.0).color(p.text));
                    });
                    ui.add_space(14.0);

                    ui.label(
                        RichText::new("Onde o clicker vai funcionar?")
                            .size(22.0)
                            .color(p.text),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(
                            "Escolha a janela em que os cliques serão enviados. \
                             Você pode trocar a qualquer momento depois.",
                        )
                        .size(13.0)
                        .color(p.muted),
                    );
                    ui.add_space(14.0);

                    // Filtro de categoria: só Minecraft ou todas as janelas.
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.mc_only, "Só Minecraft").clicked() {
                            self.mc_only = true;
                        }
                        if ui.selectable_label(!self.mc_only, "Todas as janelas").clicked() {
                            self.mc_only = false;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Atualizar").clicked() {
                                self.win_list = window_choices();
                            }
                        });
                    });
                    ui.add_space(6.0);

                    ui.add(
                        egui::TextEdit::singleline(&mut self.win_filter)
                            .hint_text("filtrar pelo título…")
                            .desired_width(col_w),
                    );
                    ui.add_space(8.0);

                    let filter = self.win_filter.to_lowercase();
                    let mc_only = self.mc_only;
                    let matches: Vec<String> = self
                        .win_list
                        .iter()
                        .filter(|t| !mc_only || minecraft_like(t))
                        .filter(|t| filter.is_empty() || t.to_lowercase().contains(&filter))
                        .cloned()
                        .collect();

                    ui.label(
                        RichText::new(format!(
                            "{} janela{}{}",
                            matches.len(),
                            if matches.len() == 1 { "" } else { "s" },
                            if mc_only { " de Minecraft" } else { "" }
                        ))
                        .size(12.0)
                        .color(p.muted),
                    );
                    ui.add_space(4.0);

                    let mut confirm_now = false;
                    theme::card(&p).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if matches.is_empty() {
                                    ui.add_space(6.0);
                                    let msg = if mc_only {
                                        "Nenhuma janela de Minecraft encontrada. Abra o jogo/launcher e clique em Atualizar, ou mude para \"Todas as janelas\"."
                                    } else {
                                        "Nenhuma janela encontrada. Abra o programa-alvo e clique em Atualizar."
                                    };
                                    ui.label(RichText::new(msg).size(12.0).color(p.muted));
                                }
                                for title in &matches {
                                    let selected = self.picked.as_deref() == Some(title.as_str());
                                    let resp = window_row(ui, &p, title, selected);
                                    if resp.clicked() {
                                        self.picked = Some(title.clone());
                                    }
                                    if resp.double_clicked() {
                                        self.picked = Some(title.clone());
                                        confirm_now = true;
                                    }
                                    ui.add_space(3.0);
                                }
                            });
                    });
                    if confirm_now {
                        self.confirm_window(true);
                    }
                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        if ui.button("Usar em qualquer janela").clicked() {
                            self.confirm_window(false);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can = self.picked.is_some();
                            let label = RichText::new("Continuar").color(p.on_accent);
                            let btn = egui::Button::new(label).fill(p.accent);
                            if ui.add_enabled(can, btn).clicked() {
                                self.confirm_window(true);
                            }
                            if let Some(t) = &self.picked {
                                let short: String = t.chars().take(32).collect();
                                ui.label(RichText::new(short).size(12.0).color(p.muted));
                            }
                        });
                    });
                });
            });
        });
    }
}

/// Cartão de seção com título.
fn section(ui: &mut egui::Ui, p: &Palette, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    section_h(ui, p, title, 0.0, body);
}

/// Cartão de seção com altura mínima (para preencher o espaço vertical).
fn section_h(
    ui: &mut egui::Ui,
    p: &Palette,
    title: &str,
    min_h: f32,
    body: impl FnOnce(&mut egui::Ui),
) {
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if min_h > 0.0 {
            ui.set_min_height(min_h);
        }
        ui.label(RichText::new(title).size(15.0).color(p.text));
        ui.add_space(8.0);
        body(ui);
    });
    ui.add_space(10.0);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.screen == Screen::Picker {
            self.pick_window(ctx);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(100));
        let p = self.pal;

        let active = self.shared.clicking.load(Ordering::SeqCst);
        let clicks = self.shared.clicks_done.load(Ordering::SeqCst);
        let cps_now = *self.shared.cps_now.lock().unwrap();

        // CPS real: cliques observados por segundo, atualizado a cada ~0,5s.
        let now = Instant::now();
        let dt = now.duration_since(self.meas_t).as_secs_f32();
        if dt >= 0.5 {
            let dc = clicks.saturating_sub(self.meas_clicks);
            self.meas_cps = if active { dc as f32 / dt } else { 0.0 };
            self.meas_clicks = clicks;
            self.meas_t = now;
        }
        if !active {
            self.meas_cps = 0.0;
        }

        let mut s: Settings = self.shared.settings.lock().unwrap().clone();
        let before = serde_json::to_string(&s).unwrap_or_default();
        let mut theme_changed = false;
        let mut go_picker = false;
        let mut toggle_req = false;

        // Captura de hotkey: aguarda a próxima tecla pressionada.
        if self.capturing {
            ctx.request_repaint();
            if let Some(vk) = crate::input::first_key_down() {
                if vk != 0x1B {
                    s.hotkey_vk = vk;
                    s.hotkey_name = vk_name(vk);
                }
                self.capturing = false;
            }
        }

        // ===================== MENU LATERAL =====================
        egui::SidePanel::left("nav")
            .exact_width(214.0)
            .resizable(false)
            .frame(egui::Frame {
                fill: p.surface,
                inner_margin: egui::Margin::symmetric(12.0, 14.0),
                stroke: egui::Stroke::new(1.0_f32, p.stroke),
                ..Default::default()
            })
            .show(ctx, |ui| {
                // Cabeçalho: logo + nome.
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(rect, egui::Rounding::same(8.0), p.accent);
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "D",
                        egui::FontId::new(19.0, egui::FontFamily::Proportional),
                        p.on_accent,
                    );
                    ui.add_space(8.0);
                    ui.label(RichText::new("DzkLP").size(18.0).color(p.text));
                });
                ui.add_space(4.0);
                // Pílula de status.
                let (txt, col) = if active {
                    ("● Ativo", GREEN)
                } else {
                    ("● Parado", p.muted)
                };
                ui.label(RichText::new(txt).size(12.0).color(col));
                ui.add_space(14.0);

                // Itens de navegação. Novos módulos entram aqui.
                if nav_item(ui, &p, "AutoClicker", self.page == Page::Clicker) {
                    self.page = Page::Clicker;
                }
                ui.add_space(2.0);
                if nav_item(ui, &p, "Perfis", self.page == Page::Profiles) {
                    self.page = Page::Profiles;
                }
                ui.add_space(2.0);
                if nav_item(ui, &p, "Aparência", self.page == Page::Appearance) {
                    self.page = Page::Appearance;
                }

                // Rodapé fixo embaixo: trocar janela + versão.
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(p.muted),
                    );
                    ui.add_space(8.0);
                    if ui
                        .add_sized([ui.available_width(), 30.0], egui::Button::new("Trocar janela"))
                        .clicked()
                    {
                        go_picker = true;
                    }
                    // Alvo atual, resumido.
                    let alvo = if s.target_window_only && !s.target_window_title.trim().is_empty() {
                        format!("Alvo: {}", ellipsize(&s.target_window_title, 22))
                    } else {
                        "Alvo: qualquer janela".to_string()
                    };
                    ui.label(RichText::new(alvo).size(11.0).color(p.muted));
                });
            });

        // ===================== CONTEÚDO =====================
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let avail = ui.available_width();
                // Ocupa a largura toda (com uma folga lateral), travando só em telas ultrawide.
                let col_w = avail.min(1500.0);
                let pad = ((avail - col_w) / 2.0).max(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.vertical(|ui| {
                        ui.set_width(col_w);
                        ui.add_space(14.0);
                        match self.page {
                            Page::Clicker => {
                                if hero(ui, &p, active, cps_now, self.meas_cps, clicks, &s) {
                                    toggle_req = true;
                                }
                                ui.add_space(12.0);
                                // Layout responsivo: 3 colunas (largo), 2+1 (médio) ou 1 (estreito).
                                // Em telas grandes os cards esticam para preencher a altura.
                                let w = ui.available_width();
                                let fill_h = (ui.available_height() - 60.0).clamp(230.0, 460.0);
                                if w >= 840.0 {
                                    ui.columns(3, |c| {
                                        clicker_speed(&mut c[0], &p, &mut s, fill_h);
                                        clicker_trigger(&mut c[1], &p, &mut s, &mut self.capturing, fill_h);
                                        clicker_advanced(&mut c[2], &p, &mut s, &self.shared, fill_h);
                                    });
                                } else if w >= 540.0 {
                                    ui.columns(2, |c| {
                                        clicker_speed(&mut c[0], &p, &mut s, fill_h);
                                        clicker_trigger(&mut c[1], &p, &mut s, &mut self.capturing, fill_h);
                                    });
                                    clicker_advanced(ui, &p, &mut s, &self.shared, 0.0);
                                } else {
                                    clicker_speed(ui, &p, &mut s, 0.0);
                                    clicker_trigger(ui, &p, &mut s, &mut self.capturing, 0.0);
                                    clicker_advanced(ui, &p, &mut s, &self.shared, 0.0);
                                }
                                ui.add_space(6.0);
                                let hint = match s.mode {
                                    1 => format!(
                                        "Segure {} para clicar; solte para parar.",
                                        s.hotkey_name
                                    ),
                                    2 => "Segure o botão esquerdo do mouse para clicar; solte para parar."
                                        .to_string(),
                                    _ => format!("Aperte {} para ligar e desligar.", s.hotkey_name),
                                };
                                ui.label(RichText::new(hint).size(12.0).color(p.muted));
                                if !self.status.is_empty() {
                                    ui.label(
                                        RichText::new(&self.status).size(12.0).color(p.accent),
                                    );
                                }
                            }
                            Page::Profiles => {
                                ui.label(RichText::new("Perfis").size(20.0).color(p.text));
                                ui.add_space(10.0);
                                self.profiles_page(ui, &p, &mut s);
                            }
                            Page::Appearance => {
                                ui.label(RichText::new("Aparência").size(20.0).color(p.text));
                                ui.add_space(10.0);
                                section(ui, &p, "Tema", |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(!s.dark_mode, "Claro").clicked() {
                                            s.dark_mode = false;
                                            theme_changed = true;
                                        }
                                        if ui.selectable_label(s.dark_mode, "Escuro").clicked() {
                                            s.dark_mode = true;
                                            theme_changed = true;
                                        }
                                    });
                                });
                            }
                        }
                        ui.add_space(8.0);
                    });
                });
            });
        });

        if theme_changed {
            self.pal = theme::palette(s.dark_mode);
            theme::install(ctx, &self.pal);
        }

        let after = serde_json::to_string(&s).unwrap_or_default();
        if before != after {
            *self.shared.settings.lock().unwrap() = s;
            self.persist();
        }

        if toggle_req {
            self.shared.ui_toggle();
        }
        if go_picker {
            self.open_picker();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.shared.running.store(false, Ordering::SeqCst);
        self.persist();
    }
}

impl App {
    /// Página de perfis (salvar/carregar/excluir configurações nomeadas).
    fn profiles_page(&mut self, ui: &mut egui::Ui, p: &Palette, s: &mut Settings) {
        section(ui, p, "Salvar configuração atual", |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_profile_name)
                        .hint_text("nome do perfil")
                        .desired_width(200.0),
                );
                if ui.button("Salvar").clicked() {
                    let name = self.new_profile_name.trim().to_string();
                    if !name.is_empty() {
                        self.config.profiles.insert(name.clone(), s.clone());
                        self.new_profile_name.clear();
                        self.status = format!("Perfil \"{name}\" salvo.");
                    }
                }
            });
        });

        section(ui, p, "Perfis salvos", |ui| {
            let names: Vec<String> = self.config.profiles.keys().cloned().collect();
            if names.is_empty() {
                ui.label(
                    RichText::new("Nenhum perfil ainda. Salve a configuração atual para reusar depois.")
                        .size(12.0)
                        .color(p.muted),
                );
            }
            for name in names {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Carregar").clicked() {
                        if let Some(prof) = self.config.profiles.get(&name) {
                            *s = prof.clone();
                            self.status = format!("Perfil \"{name}\" carregado.");
                        }
                    }
                    if ui.button(RichText::new("Excluir").color(p.danger)).clicked() {
                        self.config.profiles.remove(&name);
                    }
                    ui.label(&name);
                });
            }
        });
    }
}

/// Seção "Velocidade" da página do clicker.
fn clicker_speed(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, min_h: f32) {
    section_h(ui, p, "Velocidade", min_h, |ui| {
        // Slider adaptativo: encolhe em colunas estreitas, cresce em largas.
        let sw = (ui.available_width() - 150.0).clamp(90.0, 230.0);
        ui.spacing_mut().slider_width = sw;
        ui.add(egui::Slider::new(&mut s.min_cps, 1.0..=100.0).text("CPS mínimo"));
        ui.add_space(2.0);
        ui.add_enabled(
            s.jitter,
            egui::Slider::new(&mut s.max_cps, 1.0..=100.0).text("CPS máximo"),
        );
        if s.max_cps < s.min_cps {
            s.max_cps = s.min_cps;
        }
        ui.add_space(4.0);
        ui.checkbox(&mut s.jitter, "Variar entre mínimo e máximo");
    });
}

/// Seção "Acionamento" da página do clicker.
fn clicker_trigger(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, capturing: &mut bool, min_h: f32) {
    section_h(ui, p, "Acionamento", min_h, |ui| {
        egui::Grid::new("acion")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Botão");
                egui::ComboBox::from_id_source("btn")
                    .selected_text(crate::input::MouseButton::from_u8(s.button).label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.button, 0, "Esquerdo");
                        ui.selectable_value(&mut s.button, 1, "Direito");
                        ui.selectable_value(&mut s.button, 2, "Meio");
                    });
                ui.end_row();

                ui.label("Modo");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut s.mode, 0, "Toggle");
                    ui.selectable_value(&mut s.mode, 1, "Segurar tecla");
                    ui.selectable_value(&mut s.mode, 2, "Segurar mouse");
                });
                ui.end_row();

                ui.label("Tecla");
                ui.add_enabled_ui(s.mode != 2, |ui| {
                    let label = if *capturing {
                        "aperte uma tecla…".to_string()
                    } else {
                        vk_name(s.hotkey_vk)
                    };
                    let btn = if *capturing {
                        egui::Button::new(RichText::new(label).color(p.on_accent)).fill(p.accent)
                    } else {
                        egui::Button::new(label)
                    };
                    if ui.add(btn).clicked() {
                        *capturing = !*capturing;
                    }
                    if *capturing {
                        ui.label(RichText::new("(Esc cancela)").size(11.0).color(p.muted));
                    }
                });
                ui.end_row();
            });
    });
}

/// Seção "Avançado" da página do clicker.
fn clicker_advanced(ui: &mut egui::Ui, p: &Palette, s: &mut Settings, shared: &Arc<Shared>, min_h: f32) {
    section_h(ui, p, "Avançado", min_h, |ui| {
        ui.horizontal(|ui| {
            ui.label("Atraso ao iniciar");
            ui.add(
                egui::DragValue::new(&mut s.start_delay_ms)
                    .speed(10)
                    .clamp_range(0..=10_000)
                    .suffix(" ms"),
            );
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut s.click_limit_enabled, "Parar após");
            ui.add_enabled(
                s.click_limit_enabled,
                egui::DragValue::new(&mut s.click_limit)
                    .speed(10)
                    .clamp_range(1..=1_000_000)
                    .suffix(" cliques"),
            );
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut s.target_window_only, "Só na janela-alvo");
        });
        if s.target_window_only && !s.target_window_title.trim().is_empty() {
            ui.label(
                RichText::new(format!("Alvo: {}", ellipsize(&s.target_window_title, 40)))
                    .size(11.0)
                    .color(p.muted),
            );
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Rajada");
            ui.add(
                egui::DragValue::new(&mut s.burst_clicks)
                    .speed(1)
                    .clamp_range(1..=1000)
                    .suffix(" cliques"),
            );
            if ui.button("Disparar").clicked() {
                shared.burst_request.store(true, Ordering::SeqCst);
            }
        });
    });
}

/// Cartão de status: fica âmbar e ganha destaque quando está clicando.
/// Retorna `true` se o botão Iniciar/Parar foi clicado (modo toggle).
fn hero(
    ui: &mut egui::Ui,
    p: &Palette,
    active: bool,
    cps: f32,
    cps_meas: f32,
    clicks: u64,
    s: &Settings,
) -> bool {
    let fill = if active { p.accent } else { p.surface };
    let fg = if active { p.on_accent } else { p.text };
    let sub = if active { p.on_accent } else { p.muted };
    let mut toggled = false;

    egui::Frame {
        fill,
        rounding: egui::Rounding::same(12.0),
        stroke: egui::Stroke::new(1.0_f32, if active { p.accent } else { p.stroke }),
        inner_margin: egui::Margin::same(16.0),
        outer_margin: egui::Margin::ZERO,
        shadow: egui::epaint::Shadow::NONE,
    }
    .show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let title = if active { "Atacando" } else { "Pronto" };
                ui.label(RichText::new(title).size(22.0).color(fg));
                let caption = match s.mode {
                    1 => "Modo segurar tecla",
                    2 => "Modo segurar mouse",
                    _ => "Modo liga/desliga",
                };
                ui.label(RichText::new(caption).size(12.0).color(sub));
                if s.target_window_only && !s.target_window_title.trim().is_empty() {
                    let alvo = ellipsize(&s.target_window_title, 44);
                    ui.label(RichText::new(format!("Janela: {alvo}")).size(12.0).color(sub));
                } else {
                    ui.label(RichText::new("Janela: qualquer").size(12.0).color(sub));
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if s.mode == 0 {
                    let (txt, bg) = if active {
                        ("Parar", p.danger)
                    } else {
                        ("Iniciar", p.accent)
                    };
                    let btn = egui::Button::new(RichText::new(txt).size(15.0).color(p.on_accent))
                        .fill(bg)
                        .min_size(egui::vec2(120.0, 42.0))
                        .rounding(egui::Rounding::same(9.0));
                    if ui.add(btn).clicked() {
                        toggled = true;
                    }
                } else {
                    let hint = if s.mode == 1 {
                        "segure a tecla"
                    } else {
                        "segure o mouse"
                    };
                    ui.label(RichText::new(hint).size(12.0).color(sub));
                }
            });
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let cps_txt = if active { format!("{cps:.1}") } else { "—".to_string() };
            stat(ui, fg, sub, cps_txt, "CPS alvo");
            ui.add_space(24.0);
            let meas_txt = if active { format!("{cps_meas:.1}") } else { "—".to_string() };
            stat(ui, fg, sub, meas_txt, "CPS real");
            ui.add_space(24.0);
            stat(ui, fg, sub, format!("{clicks}"), "cliques");
        });
    });

    toggled
}

fn stat(ui: &mut egui::Ui, fg: egui::Color32, sub: egui::Color32, value: String, caption: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).monospace().size(22.0).color(fg));
        ui.label(RichText::new(caption).size(11.0).color(sub));
    });
}
