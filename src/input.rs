//! Camada de entrada: envio de cliques, leitura de teclas e janela em foco.
//! Tudo isolado aqui para o resto do app não depender diretamente da WinAPI.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            _ => MouseButton::Left,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            MouseButton::Left => "Esquerdo",
            MouseButton::Right => "Direito",
            MouseButton::Middle => "Meio",
        }
    }
}

// ---------------------------------------------------------------------------
// Implementação Windows
// ---------------------------------------------------------------------------
#[cfg(windows)]
mod imp {
    use super::MouseButton;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
        MOUSEEVENTF_RIGHTUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

    fn mouse_input(flags: MOUSE_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    pub fn send_click(btn: MouseButton) {
        let (down, up) = match btn {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };
        let inputs = [mouse_input(down), mouse_input(up)];
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// Retorna true se a tecla do virtual-key code está pressionada agora.
    pub fn key_down(vk: i32) -> bool {
        unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
    }

    /// Título da janela atualmente em foco (para o filtro de janela-alvo).
    pub fn foreground_title() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buf);
            if len <= 0 {
                return String::new();
            }
            String::from_utf16_lossy(&buf[..len as usize])
        }
    }

    /// Lista os títulos das janelas visíveis abertas (para o seletor).
    pub fn list_windows() -> Vec<String> {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
        };

        unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if IsWindowVisible(hwnd).as_bool() {
                let len = GetWindowTextLengthW(hwnd);
                if len > 0 {
                    let mut buf = vec![0u16; len as usize + 1];
                    let n = GetWindowTextW(hwnd, &mut buf);
                    if n > 0 {
                        let title = String::from_utf16_lossy(&buf[..n as usize]);
                        let v = &mut *(lparam.0 as *mut Vec<String>);
                        v.push(title);
                    }
                }
            }
            TRUE
        }

        let mut titles: Vec<String> = Vec::new();
        unsafe {
            let _ = EnumWindows(Some(cb), LPARAM(&mut titles as *mut _ as isize));
        }
        titles.sort();
        titles.dedup();
        titles
    }

    /// Primeira tecla pressionada no momento (para capturar a hotkey).
    /// Ignora o botão esquerdo do mouse (usado para clicar no campo).
    pub fn first_key_down() -> Option<i32> {
        for vk in 0x02..=0xFEi32 {
            if key_down(vk) {
                return Some(vk);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Stubs para compilar/checar fora do Windows (o alvo real é Windows)
// ---------------------------------------------------------------------------
#[cfg(not(windows))]
mod imp {
    use super::MouseButton;
    pub fn send_click(_btn: MouseButton) {}
    pub fn key_down(_vk: i32) -> bool {
        false
    }
    pub fn foreground_title() -> String {
        String::new()
    }
    pub fn list_windows() -> Vec<String> {
        Vec::new()
    }
    pub fn first_key_down() -> Option<i32> {
        None
    }
}

pub use imp::{first_key_down, foreground_title, key_down, list_windows, send_click};
