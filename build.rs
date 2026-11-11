// Embute o ícone "D" no executável no Windows (aparece no Explorer).
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("aviso: nao foi possivel embutir o icone: {e}");
        }
    }
}
