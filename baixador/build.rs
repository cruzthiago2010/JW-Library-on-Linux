fn main() {
    // Só a janela precisa do tauri-build (lê o tauri.conf.json e os ícones).
    #[cfg(feature = "janela")]
    tauri_build::build()
}
