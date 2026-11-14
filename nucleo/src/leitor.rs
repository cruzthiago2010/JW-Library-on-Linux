//! O HTML de um documento pronto para o leitor: figuras pelo protocolo do app,
//! chamadas de nota como links e links internos marcados para o JavaScript.
use std::sync::OnceLock;

use regex::Regex;

fn re(padrao: &'static str, guardado: &'static OnceLock<Regex>) -> &'static Regex {
    guardado.get_or_init(|| Regex::new(padrao).expect("regex"))
}

/// Mesmo ajuste do app Reuniões nativo (`paginas::ajustar_html`).
pub fn ajustar_html(html: &str, pasta: &str, base: &str) -> String {
    static FIGURA: OnceLock<Regex> = OnceLock::new();
    static CHAMADA: OnceLock<Regex> = OnceLock::new();
    static DOC: OnceLock<Regex> = OnceLock::new();
    static BIBLIA: OnceLock<Regex> = OnceLock::new();
    static FORA: OnceLock<Regex> = OnceLock::new();

    let html = re(r#"src="jwpub-media://([^"]+)""#, &FIGURA)
        .replace_all(html, |c: &regex::Captures| format!(r#"src="{base}/midia/{pasta}/{}""#, &c[1]));
    let html = re(
        r#"(?s)<span data-fnid="(\d+)" class="fn">(.*?)<span id="footnotesource\d+"[^>]*></span></span>"#,
        &CHAMADA,
    )
    .replace_all(&html, r##"<a id="footnotesource$1" data-fnid="$1" class="fn" href="#footnote$1">$2</a>"##);
    let html = re(r#"href="jwpub://[pc]/[A-Z]+:(\d+)/?[^"]*""#, &DOC)
        .replace_all(&html, r##"href="#" data-meps="$1""##);
    let html = re(r#"href="jwpub://b/([^"]*)""#, &BIBLIA).replace_all(&html, r##"href="#" data-biblia="$1""##);
    let html = re(r#"href="(https?://[^"]*)""#, &FORA).replace_all(&html, r##"href="#" data-fora="$1""##);
    html.into_owned()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn figura_vai_para_o_protocolo() {
        let saida = ajustar_html(r#"<img src="jwpub-media://a_cnt_1.jpg" />"#, "wcg_T", "conteudo://localhost");
        assert!(saida.contains(r#"src="conteudo://localhost/midia/wcg_T/a_cnt_1.jpg""#));
    }
}
