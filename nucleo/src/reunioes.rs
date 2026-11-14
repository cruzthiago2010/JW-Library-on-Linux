//! A aba Reuniões: para uma semana (segunda-feira), a parte da apostila
//! (`mwb`) e o artigo de estudo de A Sentinela (`w`) das publicações
//! instaladas, pela tabela `DatedText` (datas `AAAAMMDD`).
use std::path::Path;

use rusqlite::OptionalExtension;

use crate::jwpub::{self, Publicacao};

#[derive(serde::Serialize, Clone)]
pub struct Parte {
    pub pasta: String,
    pub id: i64,
    pub titulo: String,
    pub publicacao: String,
    pub miniatura: Option<String>,
    pub inicio: i64,
    pub fim: i64,
}

#[derive(serde::Serialize, Default)]
pub struct Semana {
    pub apostila: Option<Parte>,
    pub sentinela: Option<Parte>,
}

/// O documento ligado a uma linha de `DatedText`. Na Sentinela a linha aponta
/// para o índice ("Artigos"); o artigo é o link que está naqueles parágrafos.
fn documento_da_linha(p: &Publicacao, doc: i64, ini: i64, fim: i64) -> Option<i64> {
    let banco = p.banco_publico().ok()?;
    let classe: i64 = banco
        .query_row("SELECT CAST(Class AS INTEGER) FROM Document WHERE DocumentId = ?1", [doc], |l| l.get(0))
        .ok()?;
    if classe != 68 {
        return Some(doc);
    }
    let meps: Option<i64> = banco
        .query_row(
            "SELECT CAST(il.MepsDocumentId AS INTEGER) FROM DocumentInternalLink dil \
               JOIN InternalLink il ON il.InternalLinkId = dil.InternalLinkId \
              WHERE dil.DocumentId = ?1 AND dil.BeginParagraphOrdinal BETWEEN ?2 AND ?3 \
              ORDER BY dil.BeginParagraphOrdinal LIMIT 1",
            [doc, ini, fim],
            |l| l.get(0),
        )
        .optional()
        .ok()
        .flatten();
    meps.and_then(|m| p.pelo_meps(m))
}

fn miniatura(p: &Publicacao, doc: i64) -> Option<String> {
    let banco = p.banco_publico().ok()?;
    banco
        .query_row(
            "SELECT m.FilePath FROM DocumentMultimedia dm JOIN Multimedia m ON m.MultimediaId = dm.MultimediaId \
              WHERE dm.DocumentId = ?1 AND m.CategoryType IN (9, 8) ORDER BY m.CategoryType DESC LIMIT 1",
            [doc],
            |l| l.get(0),
        )
        .optional()
        .ok()
        .flatten()
}

/// A parte de uma publicação que cobre o dia `data` (AAAAMMDD).
fn parte(p: &Publicacao, data: i64) -> Option<Parte> {
    let banco = p.banco_publico().ok()?;
    let (doc, ini, fim, de, ate): (i64, i64, i64, i64, i64) = banco
        .query_row(
            "SELECT DocumentId, IFNULL(BeginParagraphOrdinal, 0), IFNULL(EndParagraphOrdinal, 0), \
                    CAST(FirstDateOffset AS INTEGER), CAST(LastDateOffset AS INTEGER) \
               FROM DatedText WHERE CAST(FirstDateOffset AS INTEGER) <= ?1 AND CAST(LastDateOffset AS INTEGER) >= ?1 \
              LIMIT 1",
            [data],
            |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?, l.get(3)?, l.get(4)?)),
        )
        .optional()
        .ok()
        .flatten()?;
    let id = documento_da_linha(p, doc, ini, fim)?;
    let titulo = p.documento(id).ok()?.titulo;
    Some(Parte {
        miniatura: miniatura(p, id),
        pasta: p.pasta.clone(),
        publicacao: p.titulo.clone(),
        id,
        titulo,
        inicio: de,
        fim: ate,
    })
}

/// A semana que contém `data`, com o que estiver instalado.
pub fn semana(raiz: &Path, data: i64) -> Semana {
    let mut s = Semana::default();
    for p in jwpub::instaladas(raiz) {
        match p.sigla.as_str() {
            "mwb" if s.apostila.is_none() => s.apostila = parte(&p, data),
            "w" if s.sentinela.is_none() => s.sentinela = parte(&p, data),
            _ => {}
        }
    }
    s
}
