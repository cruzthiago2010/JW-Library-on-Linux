//! Idioma da interface (português ou inglês): nomes de categorias e de livros
//! da Bíblia que o núcleo entrega para a janela. As publicações não mudam.
use std::sync::atomic::{AtomicBool, Ordering};

static INGLES: AtomicBool = AtomicBool::new(false);

pub fn definir(idioma: &str) {
    INGLES.store(idioma == "en", Ordering::Relaxed);
}

pub fn ingles() -> bool {
    INGLES.load(Ordering::Relaxed)
}

/// O texto em português ou em inglês, conforme o idioma escolhido.
pub fn t(pt: &'static str, en: &'static str) -> &'static str {
    if ingles() { en } else { pt }
}
