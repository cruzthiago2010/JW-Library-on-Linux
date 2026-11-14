//! Os tokens de um parágrafo, como o JW Library conta (`StartToken`/`EndToken`
//! das marcas). Regra trazida do app Reuniões (`app/text/tokenizer.py` e
//! `wol.texto_para_tokens`): base 0, fim inclusivo, o número do parágrafo não
//! conta; pontuação no começo e no fim da palavra vira token próprio; dois nós
//! de texto colados (sem espaço) são tokens separados. A mesma regra está em
//! `ui/marcas.js`, que é quem pinta na tela.
use std::sync::OnceLock;

use regex::Regex;

pub const PONTUACAO: &str = ".,;:!?()[]“”‘’\"'—–…/";
pub const LIMITE: char = '¦';

/// O texto de um parágrafo, com `¦` nos limites de elemento e sem o que não
/// conta (número do parágrafo/versículo, letras de referência, campos).
pub fn texto_do_paragrafo(html: &str) -> String {
    static ETIQUETA: OnceLock<Regex> = OnceLock::new();
    let re = ETIQUETA.get_or_init(|| Regex::new(r#"(?s)<(/?)([a-zA-Z0-9]+)([^>]*?)(/?)>"#).unwrap());
    let mut pedacos: Vec<String> = vec![];
    let mut pulando = 0usize;
    let mut cursor = 0;
    let pular = |attrs: &str| {
        attrs.contains("parNum") || attrs.contains("class=\"vl\"") || attrs.contains("class=\"cl\"")
            || attrs.contains("class=\"m\"") || attrs.contains("class=\"tt ") || attrs.contains("gen-field")
    };
    let juntar = |texto: &str, pedacos: &mut Vec<String>| {
        if texto.is_empty() {
            return;
        }
        let texto = texto.replace("&nbsp;", "\u{a0}").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"");
        if let Some(ultimo) = pedacos.last() {
            if !ultimo.is_empty() && !ultimo.ends_with(char::is_whitespace) && !texto.starts_with(char::is_whitespace) {
                pedacos.push(LIMITE.to_string());
            }
        }
        pedacos.push(texto);
    };
    for m in re.captures_iter(html) {
        let todo = m.get(0).unwrap();
        if pulando == 0 {
            juntar(&html[cursor..todo.start()], &mut pedacos);
        }
        cursor = todo.end();
        let fecha = &m[1] == "/";
        let sozinha = &m[4] == "/" || matches!(&m[2], "br" | "img" | "hr" | "input" | "wbr");
        if sozinha {
            continue;
        }
        if pulando > 0 {
            if fecha { pulando -= 1 } else { pulando += 1 }
        } else if !fecha && pular(&m[3]) {
            pulando = 1;
        }
    }
    if pulando == 0 {
        juntar(&html[cursor..], &mut pedacos);
    }
    let junto = pedacos.concat();
    junto.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// As posições (em caracteres) de cada token no texto.
pub fn tokenizar(texto: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = texto.chars().collect();
    let mut saida = vec![];
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() || chars[i] == LIMITE {
            i += 1;
            continue;
        }
        let base = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != LIMITE {
            i += 1;
        }
        let (mut ini, mut fim) = (base, i);
        while ini < fim && PONTUACAO.contains(chars[ini]) {
            saida.push((ini, ini + 1));
            ini += 1;
        }
        let mut cauda = vec![];
        while fim > ini && PONTUACAO.contains(chars[fim - 1]) {
            fim -= 1;
            cauda.push((fim, fim + 1));
        }
        if ini < fim {
            saida.push((ini, fim));
        }
        saida.extend(cauda.into_iter().rev());
    }
    saida
}

/// O trecho dos tokens `inicio..=fim` do texto.
pub fn trecho(texto: &str, inicio: usize, fim: usize) -> Option<String> {
    let t = tokenizar(texto);
    let (a, _) = *t.get(inicio)?;
    let (_, b) = *t.get(fim.min(t.len().saturating_sub(1)))?;
    Some(texto.chars().skip(a).take(b - a).collect::<String>().replace(LIMITE, ""))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn regras_do_jw_library() {
        let t = "“Jeová, adorá-lo”, disse ele.";
        let tk: Vec<String> = tokenizar(t).iter().map(|&(a, b)| t.chars().skip(a).take(b - a).collect()).collect();
        assert_eq!(tk, ["“", "Jeová", ",", "adorá-lo", "”", ",", "disse", "ele", "."]);
        assert_eq!(texto_do_paragrafo(r#"<span class="parNum">3</span> <em>A</em>me o próximo"#), "A¦me o próximo");
        assert_eq!(trecho("A¦me o próximo", 1, 2).as_deref(), Some("me o"));
    }
}
