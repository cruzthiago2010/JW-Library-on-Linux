//! O painel de estudo de um documento: notas de rodapé e trechos de outras
//! publicações citados (tabelas `Footnote` e `Extract`), por parágrafo.
use std::sync::OnceLock;

use regex::Regex;

use crate::jwpub::{Publicacao, Resultado};
use crate::leitor::ajustar_html;

#[derive(serde::Serialize)]
pub struct Nota {
    /// O `data-fnid` da chamada no texto.
    pub indice: i64,
    pub paragrafo: i64,
    pub html: String,
}

#[derive(serde::Serialize)]
pub struct Trecho {
    pub paragrafo: i64,
    /// Referência da origem ("w26 dezembro pág. 3") e título, já em HTML simples.
    pub origem: String,
    pub html: String,
}

#[derive(serde::Serialize)]
pub struct Painel {
    pub notas: Vec<Nota>,
    pub trechos: Vec<Trecho>,
}

/// Tira imagens (de outras publicações, que não estão aqui) e deixa o resto.
fn sem_imagens(html: &str) -> String {
    static IMG: OnceLock<Regex> = OnceLock::new();
    IMG.get_or_init(|| Regex::new(r"(?s)<img[^>]*>|<figure.*?</figure>").unwrap()).replace_all(html, "").into_owned()
}

/// Só as marcas de texto que a legenda traz (`<span class="eloc">`…): o resto vira texto.
fn legenda(rica: &str) -> String {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let limpo = TAG.get_or_init(|| Regex::new(r"<[^>]+>").unwrap()).replace_all(rica, " · ");
    limpo.split(" · ").map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ")
}

pub fn painel(p: &Publicacao, documento: i64, base: &str) -> Resultado<Painel> {
    let banco = p.banco_publico()?;
    let mut notas = Vec::new();
    if let Ok(mut c) = banco.prepare(
        "SELECT FootnoteIndex, IFNULL(ParagraphOrdinal, 0), Content FROM Footnote \
         WHERE DocumentId = ?1 ORDER BY FootnoteIndex",
    ) {
        let linhas = c.query_map([documento], |l| Ok((l.get::<_, i64>(0)?, l.get::<_, i64>(1)?, l.get::<_, Option<Vec<u8>>>(2)?)));
        for (indice, paragrafo, bruto) in linhas.map_err(|e| e.to_string())?.filter_map(Result::ok) {
            let html = match bruto {
                Some(b) if !b.is_empty() => p.claro(&b).unwrap_or_default(),
                _ => String::new(),
            };
            notas.push(Nota { indice, paragrafo, html: ajustar_html(&html, &p.pasta, base) });
        }
    }
    let mut trechos = Vec::new();
    if let Ok(mut c) = banco.prepare(
        "SELECT IFNULL(de.BeginParagraphOrdinal, 0), IFNULL(e.CaptionRich, e.Caption), e.Content \
           FROM DocumentExtract de JOIN Extract e ON e.ExtractId = de.ExtractId \
          WHERE de.DocumentId = ?1 ORDER BY de.BeginParagraphOrdinal, de.SortPosition",
    ) {
        let linhas = c.query_map([documento], |l| {
            Ok((l.get::<_, i64>(0)?, l.get::<_, Option<String>>(1)?, l.get::<_, Option<Vec<u8>>>(2)?))
        });
        for (paragrafo, rica, bruto) in linhas.map_err(|e| e.to_string())?.filter_map(Result::ok) {
            let html = match bruto {
                Some(b) if !b.is_empty() => p.claro(&b).unwrap_or_default(),
                _ => String::new(),
            };
            trechos.push(Trecho {
                paragrafo,
                origem: legenda(rica.as_deref().unwrap_or("")),
                html: ajustar_html(&sem_imagens(&html), &p.pasta, base),
            });
        }
    }
    Ok(Painel { notas, trechos })
}

#[cfg(test)]
mod testes {
    #[test]
    fn legenda_vira_texto() {
        let r = super::legenda(r#"<span class="eloc">w26 dezembro pág. 3</span> <span class="etitle">Artigos</span>"#);
        assert_eq!(r, "w26 dezembro pág. 3 · Artigos");
    }
}
