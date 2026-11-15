//! Busca na Bíblia, como a lupa do JW Library: um índice FTS5 do SQLite
//! (`<dados>/busca.db`) com um versículo por linha, montado na primeira busca
//! e refeito quando a Bíblia instalada muda. Ignora acentos e maiúsculas
//! ("jeova" acha "Jeová"). Adaptado do app Reuniões nativo.
use std::path::Path;

use regex::Regex;
use rusqlite::{params, Connection};

use crate::biblia::{rotulo, Biblia};

pub type Resultado<T> = Result<T, String>;

#[derive(serde::Serialize)]
pub struct Achado {
    pub codigo: u32,
    pub rotulo: String,
    pub trecho: String,
}

fn sql(e: rusqlite::Error) -> String {
    format!("busca: {e}")
}

/// O texto puro de um HTML.
fn texto_puro(html: &str) -> String {
    static MARCAS: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let marcas = MARCAS.get_or_init(|| Regex::new(r"(?s)<[^>]*>").unwrap());
    let texto = marcas.replace_all(html, " ").replace("&nbsp;", " ");
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn abrir(caminho: &Path) -> Resultado<Connection> {
    let c = Connection::open(caminho).map_err(sql)?;
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS Estado(Chave TEXT PRIMARY KEY, Valor TEXT);
         CREATE VIRTUAL TABLE IF NOT EXISTS Versos USING fts5(
             codigo UNINDEXED, texto, tokenize = 'unicode61 remove_diacritics 2');
         CREATE VIRTUAL TABLE IF NOT EXISTS Docs USING fts5(
             pasta UNINDEXED, id UNINDEXED, publicacao UNINDEXED, titulo, texto,
             tokenize = 'unicode61 remove_diacritics 2');",
    )
    .map_err(sql)?;
    Ok(c)
}

/// Monta (ou refaz) o índice se a Bíblia mudou. Devolve se refez.
pub fn preparar(biblioteca: &Path, caminho: &Path) -> Resultado<bool> {
    let Some(b) = Biblia::achar(biblioteca) else { return Ok(false) };
    let agora = format!("{}:{}", b.publicacao.pasta, b.publicacao.ano);
    let c = abrir(caminho)?;
    let antes: Option<String> = c.query_row("SELECT Valor FROM Estado WHERE Chave='biblia'", [], |l| l.get(0)).ok();
    if antes.as_deref() == Some(agora.as_str()) {
        return Ok(false);
    }
    c.execute_batch("BEGIN; DELETE FROM Versos;").map_err(sql)?;
    {
        let mut inserir = c.prepare("INSERT INTO Versos (codigo, texto) VALUES (?, ?)").map_err(sql)?;
        for livro in b.livros()? {
            let inicio = livro.numero * 1_000_000 + 1_001;
            let fim = livro.numero * 1_000_000 + 999_999;
            let Ok(versos) = b.versiculos(inicio, fim) else { continue };
            for v in versos {
                let codigo = v.livro * 1_000_000 + v.capitulo * 1_000 + v.verso;
                inserir.execute(params![codigo, texto_puro(&v.html)]).map_err(sql)?;
            }
        }
    }
    c.execute("INSERT OR REPLACE INTO Estado (Chave, Valor) VALUES ('biblia', ?)", [&agora]).map_err(sql)?;
    c.execute_batch("COMMIT;").map_err(sql)?;
    Ok(true)
}

/// As palavras como consulta FTS5: todas precisam estar; com `exata`, a
/// expressão inteira; sem ela, a última vale como começo de palavra.
fn consulta(termos: &str, exata: bool) -> Option<String> {
    let palavras: Vec<String> =
        termos.split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty()).map(|p| p.replace('"', "")).collect();
    if palavras.is_empty() {
        return None;
    }
    if exata {
        return Some(format!("\"{}\"", palavras.join(" ")));
    }
    let mut q: Vec<String> = palavras.iter().map(|p| format!("\"{p}\"")).collect();
    let ultima = q.pop()?;
    q.push(format!("{ultima}*"));
    Some(q.join(" "))
}

/// Os versículos que têm os termos, na ordem da Bíblia, e o total.
pub fn buscar(biblioteca: &Path, caminho: &Path, termos: &str, exata: bool, limite: usize) -> Resultado<(usize, Vec<Achado>)> {
    preparar(biblioteca, caminho)?;
    let Some(q) = consulta(termos, exata) else { return Ok((0, vec![])) };
    let c = abrir(caminho)?;
    let total: i64 = c.query_row("SELECT count(*) FROM Versos WHERE Versos MATCH ?", [&q], |l| l.get(0)).map_err(sql)?;
    let mut s = c
        .prepare(
            "SELECT CAST(codigo AS INTEGER), highlight(Versos, 1, '<b>', '</b>') FROM Versos \
             WHERE Versos MATCH ? ORDER BY CAST(codigo AS INTEGER) LIMIT ?",
        )
        .map_err(sql)?;
    let linhas = s
        .query_map(params![q, limite as i64], |l| {
            let codigo: u32 = l.get(0)?;
            Ok(Achado { codigo, rotulo: rotulo(codigo, codigo), trecho: l.get(1)? })
        })
        .map_err(sql)?;
    Ok((total as usize, linhas.collect::<Result<_, _>>().map_err(sql)?))
}

#[derive(serde::Serialize)]
pub struct AchadoPub {
    pub pasta: String,
    pub id: i64,
    pub titulo: String,
    pub publicacao: String,
    pub trecho: String,
}

/// A "assinatura" das publicações instaladas (menos as Bíblias): muda quando
/// alguma entra, sai ou é atualizada.
fn assinatura(biblioteca: &Path) -> (String, Vec<crate::jwpub::Publicacao>) {
    let pubs: Vec<_> = crate::jwpub::instaladas(biblioteca).into_iter().filter(|p| !p.simbolo.starts_with("nwt")).collect();
    let texto = pubs
        .iter()
        .map(|p| {
            let marca = std::fs::read_to_string(biblioteca.join(&p.pasta).join("instalado.json")).unwrap_or_default();
            format!("{}={}", p.pasta, marca.trim())
        })
        .collect::<Vec<_>>()
        .join(";");
    (texto, pubs)
}

/// Monta (ou refaz) o índice das publicações. Devolve se refez.
pub fn preparar_publicacoes(biblioteca: &Path, caminho: &Path) -> Resultado<bool> {
    let (agora, pubs) = assinatura(biblioteca);
    let c = abrir(caminho)?;
    let antes: Option<String> = c.query_row("SELECT Valor FROM Estado WHERE Chave='publicacoes'", [], |l| l.get(0)).ok();
    if antes.as_deref() == Some(agora.as_str()) {
        return Ok(false);
    }
    c.execute_batch("BEGIN; DELETE FROM Docs;").map_err(sql)?;
    {
        let mut inserir = c.prepare("INSERT INTO Docs (pasta, id, publicacao, titulo, texto) VALUES (?, ?, ?, ?, ?)").map_err(sql)?;
        for p in &pubs {
            let Ok(ids) = p.documentos() else { continue };
            for id in ids {
                let Ok(d) = p.documento(id) else { continue };
                let texto = texto_puro(&d.html);
                if texto.is_empty() {
                    continue;
                }
                inserir.execute(params![p.pasta, id, p.titulo, d.titulo, texto]).map_err(sql)?;
            }
        }
    }
    c.execute("INSERT OR REPLACE INTO Estado (Chave, Valor) VALUES ('publicacoes', ?)", [&agora]).map_err(sql)?;
    c.execute_batch("COMMIT;").map_err(sql)?;
    Ok(true)
}

/// Os artigos e capítulos das publicações (menos a Bíblia) que têm os termos,
/// os mais relevantes primeiro, e o total.
pub fn buscar_publicacoes(biblioteca: &Path, caminho: &Path, termos: &str, exata: bool, limite: usize) -> Resultado<(usize, Vec<AchadoPub>)> {
    preparar_publicacoes(biblioteca, caminho)?;
    let Some(q) = consulta(termos, exata) else { return Ok((0, vec![])) };
    let c = abrir(caminho)?;
    let total: i64 = c.query_row("SELECT count(*) FROM Docs WHERE Docs MATCH ?", [&q], |l| l.get(0)).map_err(sql)?;
    let mut s = c
        .prepare(
            "SELECT pasta, CAST(id AS INTEGER), publicacao, titulo, snippet(Docs, 4, '<b>', '</b>', '…', 24) FROM Docs \
             WHERE Docs MATCH ? ORDER BY bm25(Docs, 0, 0, 0, 5.0, 1.0) LIMIT ?",
        )
        .map_err(sql)?;
    let linhas = s
        .query_map(params![q, limite as i64], |l| {
            Ok(AchadoPub { pasta: l.get(0)?, id: l.get(1)?, publicacao: l.get(2)?, titulo: l.get(3)?, trecho: l.get(4)? })
        })
        .map_err(sql)?;
    Ok((total as usize, linhas.collect::<Result<_, _>>().map_err(sql)?))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn consulta_com_prefixo_ou_exata() {
        assert_eq!(consulta("Jeová cora", false).as_deref(), Some("\"Jeová\" \"cora\"*"));
        assert_eq!(consulta("nascer de novo", true).as_deref(), Some("\"nascer de novo\""));
        assert_eq!(consulta("  ", false), None);
    }
}
