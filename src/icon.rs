//! Gera o ícone do app (logo "D") em RGBA, com antisserrilhamento,
//! para usar como ícone da janela e da barra de tarefas.

/// Fundo em ardósia arredondado com um "D" âmbar.
pub fn build() -> egui::IconData {
    let size: usize = 128;
    let ss: usize = 3; // supersampling por eixo
    let mut rgba = vec![0u8; size * size * 4];

    // Cores (ardósia e âmbar, iguais ao tema).
    let (sr, sg, sb) = (0x1C as f32, 0x1F as f32, 0x27 as f32);
    let (ar, ag, ab) = (0xF5 as f32, 0xA5 as f32, 0x24 as f32);

    for py in 0..size {
        for px in 0..size {
            let mut bg = 0.0f32;
            let mut gl = 0.0f32;
            for sy in 0..ss {
                for sx in 0..ss {
                    let fx = (px as f32 + (sx as f32 + 0.5) / ss as f32) / size as f32;
                    let fy = (py as f32 + (sy as f32 + 0.5) / ss as f32) / size as f32;
                    if in_rounded_rect(fx, fy) {
                        bg += 1.0;
                    }
                    if in_glyph(fx, fy) {
                        gl += 1.0;
                    }
                }
            }
            let n = (ss * ss) as f32;
            let bg = bg / n;
            let gl = gl / n;

            // Glyph só aparece sobre o fundo.
            let mix = gl * bg;
            let cr = sr * (1.0 - mix) + ar * mix;
            let cg = sg * (1.0 - mix) + ag * mix;
            let cb = sb * (1.0 - mix) + ab * mix;

            let idx = (py * size + px) * 4;
            rgba[idx] = cr as u8;
            rgba[idx + 1] = cg as u8;
            rgba[idx + 2] = cb as u8;
            rgba[idx + 3] = (bg * 255.0) as u8;
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

fn in_rounded_rect(x: f32, y: f32) -> bool {
    let m = 0.06;
    let r = 0.20;
    let lo = m;
    let hi = 1.0 - m;
    if x < lo || x > hi || y < lo || y > hi {
        return false;
    }
    let cx = x.clamp(lo + r, hi - r);
    let cy = y.clamp(lo + r, hi - r);
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
}

/// "D": haste reta à esquerda e barriga arredondada à direita, com vazado.
fn in_glyph(x: f32, y: f32) -> bool {
    let x0 = 0.32;
    let cy = 0.50;
    let rx = 0.36;
    let ry = 0.26;

    let outer = x >= x0 && {
        let a = (x - x0) / rx;
        let b = (y - cy) / ry;
        a * a + b * b <= 1.0
    };

    let x0_in = x0 + 0.12;
    let rx_in = 0.17;
    let ry_in = 0.16;
    let inner = x >= x0_in && {
        let a = (x - x0_in) / rx_in;
        let b = (y - cy) / ry_in;
        a * a + b * b <= 1.0
    };

    outer && !inner
}
