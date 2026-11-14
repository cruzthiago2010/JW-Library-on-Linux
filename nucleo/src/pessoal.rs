//! Estudo pessoal: grifos, notas, etiquetas e marcadores, guardados num banco
//! com o MESMO esquema do `userData.db` do JW Library (versão 16, `userdata.sql`).
//! Assim o backup `.jwlibrary` (Etapa 5) é este arquivo dentro de um ZIP.
//!
//! Como o JW Library grava (conferido num backup real do aparelho):
//! - documento: `Location` com `DocumentId` = MepsDocumentId, `KeySymbol`
//!   (sigla, ex. "w"), `IssueTagNumber`, `MepsLanguage`, `Type` 0;
//!   bloco = parágrafo (`BlockType` 1, `Identifier` = data-pid);
//! - Bíblia: `Location` com `BookNumber`, `ChapterNumber`, `KeySymbol`
//!   ("nwtsty"), `Type` 0; bloco = versículo (`BlockType` 2, `Identifier` = nº);
//! - grifo: `UserMark` (cor 1–6, GUID maiúsculo) + uma `BlockRange` por bloco,
//!   com `StartToken`/`EndToken` (base 0, fim inclusivo);
//! - marcador: `Bookmark` com `Slot` 0–9 por publicação (`PublicationLocationId`
//!   aponta para a `Location` Type 1 da publicação).
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::jwpub::Publicacao;

pub type Resultado<T> = Result<T, String>;

const ESQUEMA: &str = include_str!("userdata.sql");
pub const VERSAO_DO_ESQUEMA: i64 = 16;

fn sql(e: rusqlite::Error) -> String {
    format!("estudo pessoal: {e}")
}

pub fn novo_guid() -> String {
    uuid::Uuid::new_v4().to_string().to_uppercase()
}

/// Onde a marca está: um documento de publicação ou um capítulo da Bíblia.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(tag = "tipo", rename_all = "lowercase")]
pub enum Alvo {
    Doc { pasta: String, id: i64 },
    Biblia { livro: u32, capitulo: u32 },
}

/// A identidade que vai na `Location`.
#[derive(Clone, Debug)]
pub struct Identidade {
    pub meps: Option<i64>,
    pub livro: Option<u32>,
    pub capitulo: Option<u32>,
    pub simbolo: String,
    pub edicao: i64,
    pub lingua: i64,
    pub titulo: String,
}

impl Identidade {
    pub fn de_documento(p: &Publicacao, id: i64) -> Resultado<Self> {
        let d = p.documento(id)?;
        Ok(Self { meps: Some(d.meps), livro: None, capitulo: None, simbolo: p.sigla.clone(), edicao: p.edicao, lingua: p.lingua, titulo: d.titulo })
    }
    pub fn de_capitulo(biblia: &Publicacao, livro: u32, capitulo: u32, titulo: String) -> Self {
        Self { meps: None, livro: Some(livro), capitulo: Some(capitulo), simbolo: biblia.simbolo.clone(), edicao: 0, lingua: biblia.lingua, titulo }
    }
    fn bloco(&self) -> i64 {
        if self.livro.is_some() { 2 } else { 1 }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Faixa {
    /// Parágrafo (data-pid) ou versículo.
    pub bloco: i64,
    pub inicio: i64,
    pub fim: i64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Grifo {
    pub guid: String,
    pub cor: i64,
    pub faixas: Vec<Faixa>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Nota {
    pub guid: String,
    pub titulo: String,
    pub texto: String,
    /// Parágrafo ou versículo a que a nota está presa (sem = nota do documento).
    pub bloco: Option<i64>,
    pub grifo: Option<String>,
    pub cor: i64,
    pub etiquetas: Vec<String>,
    pub alterada: String,
}

#[derive(serde::Serialize, Default)]
pub struct DoLugar {
    pub grifos: Vec<Grifo>,
    pub notas: Vec<Nota>,
    pub campos: Vec<(String, String)>,
}

/// Uma nota na tela "Notas e etiquetas", com a origem para abrir.
#[derive(serde::Serialize)]
pub struct NotaComOrigem {
    #[serde(flatten)]
    pub nota: Nota,
    pub origem: String,
    pub simbolo: String,
    pub meps: Option<i64>,
    pub livro: Option<u32>,
    pub capitulo: Option<u32>,
    pub edicao: i64,
}

#[derive(serde::Serialize)]
pub struct Marcador {
    pub slot: i64,
    pub titulo: String,
    pub trecho: String,
    pub meps: Option<i64>,
    pub livro: Option<u32>,
    pub capitulo: Option<u32>,
    pub bloco: Option<i64>,
}

pub struct Pessoal {
    pub conexao: Connection,
}

impl Pessoal {
    /// Abre (ou cria vazio, com o esquema do JW Library) o banco do usuário.
    pub fn abrir(caminho: &Path) -> Resultado<Self> {
        let novo = !caminho.exists();
        let conexao = Connection::open(caminho).map_err(sql)?;
        if novo {
            criar_vazio(&conexao)?;
        }
        conexao.execute_batch("PRAGMA foreign_keys = ON;").map_err(sql)?;
        Ok(Self { conexao })
    }

    fn local(&self, id: &Identidade) -> Resultado<i64> {
        let achado: Option<i64> = if let (Some(l), Some(c)) = (id.livro, id.capitulo) {
            self.conexao
                .query_row(
                    "SELECT LocationId FROM Location WHERE BookNumber=? AND ChapterNumber=? AND KeySymbol=? \
                     AND MepsLanguage=? AND Type=0",
                    params![l, c, id.simbolo, id.lingua],
                    |r| r.get(0),
                )
                .optional()
                .map_err(sql)?
        } else {
            self.conexao
                .query_row(
                    "SELECT LocationId FROM Location WHERE DocumentId=? AND MepsLanguage=? AND Type=0 \
                     AND BookNumber IS NULL ORDER BY LocationId LIMIT 1",
                    params![id.meps, id.lingua],
                    |r| r.get(0),
                )
                .optional()
                .map_err(sql)?
        };
        if let Some(l) = achado {
            return Ok(l);
        }
        self.conexao
            .execute(
                "INSERT INTO Location (BookNumber, ChapterNumber, DocumentId, IssueTagNumber, KeySymbol, MepsLanguage, Type, Title) \
                 VALUES (?, ?, ?, ?, ?, ?, 0, ?)",
                params![id.livro, id.capitulo, id.meps, id.edicao, id.simbolo, id.lingua, id.titulo],
            )
            .map_err(sql)?;
        Ok(self.conexao.last_insert_rowid())
    }

    fn local_existente(&self, id: &Identidade) -> Resultado<Option<i64>> {
        if let (Some(l), Some(c)) = (id.livro, id.capitulo) {
            self.conexao
                .query_row(
                    "SELECT LocationId FROM Location WHERE BookNumber=? AND ChapterNumber=? AND KeySymbol=? AND MepsLanguage=? AND Type=0",
                    params![l, c, id.simbolo, id.lingua],
                    |r| r.get(0),
                )
                .optional()
                .map_err(sql)
        } else {
            self.conexao
                .query_row(
                    "SELECT LocationId FROM Location WHERE DocumentId=? AND MepsLanguage=? AND Type=0 AND BookNumber IS NULL ORDER BY LocationId LIMIT 1",
                    params![id.meps, id.lingua],
                    |r| r.get(0),
                )
                .optional()
                .map_err(sql)
        }
    }

    /// A `Location` Type 1 da publicação (a dos marcadores).
    fn local_da_publicacao(&self, id: &Identidade) -> Resultado<i64> {
        let achado: Option<i64> = self
            .conexao
            .query_row(
                "SELECT LocationId FROM Location WHERE Type=1 AND KeySymbol=? AND IssueTagNumber=? AND MepsLanguage=?",
                params![id.simbolo, id.edicao, id.lingua],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        if let Some(l) = achado {
            return Ok(l);
        }
        self.conexao
            .execute(
                "INSERT INTO Location (IssueTagNumber, KeySymbol, MepsLanguage, Type) VALUES (?, ?, ?, 1)",
                params![id.edicao, id.simbolo, id.lingua],
            )
            .map_err(sql)?;
        Ok(self.conexao.last_insert_rowid())
    }

    /// Grifa (ou recolore/redesenha, com o mesmo `guid`) uma ou mais faixas.
    pub fn grifar(&self, id: &Identidade, faixas: &[Faixa], cor: i64, guid: Option<String>) -> Resultado<String> {
        if !(1..=6).contains(&cor) {
            return Err("cor inválida".into());
        }
        if faixas.is_empty() || faixas.iter().any(|f| f.inicio < 0 || f.fim < f.inicio) {
            return Err("faixa inválida".into());
        }
        let guid = guid.filter(|g| !g.is_empty()).map(|g| g.to_uppercase()).unwrap_or_else(novo_guid);
        let loc = self.local(id)?;
        let existente: Option<i64> = self
            .conexao
            .query_row("SELECT UserMarkId FROM UserMark WHERE UserMarkGuid=?", [&guid], |r| r.get(0))
            .optional()
            .map_err(sql)?;
        let marca = match existente {
            Some(m) => {
                self.conexao
                    .execute("UPDATE UserMark SET ColorIndex=?, LocationId=?, Version=Version+1 WHERE UserMarkId=?", params![cor, loc, m])
                    .map_err(sql)?;
                self.conexao.execute("DELETE FROM BlockRange WHERE UserMarkId=?", [m]).map_err(sql)?;
                m
            }
            None => {
                self.conexao
                    .execute(
                        "INSERT INTO UserMark (ColorIndex, LocationId, StyleIndex, UserMarkGuid, Version) VALUES (?, ?, 0, ?, 1)",
                        params![cor, loc, guid],
                    )
                    .map_err(sql)?;
                self.conexao.last_insert_rowid()
            }
        };
        for f in faixas {
            self.conexao
                .execute(
                    "INSERT INTO BlockRange (BlockType, Identifier, StartToken, EndToken, UserMarkId) VALUES (?, ?, ?, ?, ?)",
                    params![id.bloco(), f.bloco, f.inicio, f.fim, marca],
                )
                .map_err(sql)?;
        }
        Ok(guid)
    }

    /// Tira um grifo. Nota presa a ele continua, solta no mesmo bloco.
    pub fn remover_grifo(&self, guid: &str) -> Resultado<()> {
        let Some(marca): Option<i64> = self
            .conexao
            .query_row("SELECT UserMarkId FROM UserMark WHERE UserMarkGuid=?", [guid], |r| r.get(0))
            .optional()
            .map_err(sql)?
        else {
            return Ok(());
        };
        self.conexao.execute("UPDATE Note SET UserMarkId=NULL WHERE UserMarkId=?", [marca]).map_err(sql)?;
        self.conexao.execute("DELETE FROM BlockRange WHERE UserMarkId=?", [marca]).map_err(sql)?;
        self.conexao.execute("DELETE FROM UserMark WHERE UserMarkId=?", [marca]).map_err(sql)?;
        Ok(())
    }

    /// Cria ou atualiza uma nota. `bloco` = parágrafo/versículo; `grifo` = GUID do grifo.
    #[allow(clippy::too_many_arguments)]
    pub fn anotar(&self, id: &Identidade, guid: Option<String>, bloco: Option<i64>, grifo: Option<&str>, titulo: &str, texto: &str, etiquetas: &[String]) -> Resultado<String> {
        let guid = guid.filter(|g| !g.is_empty()).map(|g| g.to_uppercase()).unwrap_or_else(novo_guid);
        let loc = self.local(id)?;
        let marca: Option<i64> = match grifo {
            Some(g) => self.conexao.query_row("SELECT UserMarkId FROM UserMark WHERE UserMarkGuid=?", [g], |r| r.get(0)).optional().map_err(sql)?,
            None => None,
        };
        let (tipo, ident) = match bloco {
            Some(b) => (id.bloco(), Some(b)),
            None => (0, None),
        };
        let existe: Option<i64> = self.conexao.query_row("SELECT NoteId FROM Note WHERE Guid=?", [&guid], |r| r.get(0)).optional().map_err(sql)?;
        let nota = match existe {
            Some(n) => {
                self.conexao
                    .execute(
                        "UPDATE Note SET LocationId=?, UserMarkId=?, Title=?, Content=?, BlockType=?, BlockIdentifier=?, \
                         LastModified=strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE NoteId=?",
                        params![loc, marca, titulo, texto, tipo, ident, n],
                    )
                    .map_err(sql)?;
                n
            }
            None => {
                self.conexao
                    .execute(
                        "INSERT INTO Note (Guid, UserMarkId, LocationId, Title, Content, BlockType, BlockIdentifier) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        params![guid, marca, loc, titulo, texto, tipo, ident],
                    )
                    .map_err(sql)?;
                self.conexao.last_insert_rowid()
            }
        };
        self.etiquetar(nota, etiquetas)?;
        Ok(guid)
    }

    /// As etiquetas de uma nota passam a ser exatamente estas (cria as que faltam).
    fn etiquetar(&self, nota: i64, etiquetas: &[String]) -> Resultado<()> {
        self.conexao.execute("DELETE FROM TagMap WHERE NoteId=?", [nota]).map_err(sql)?;
        for nome in etiquetas.iter().map(|e| e.trim()).filter(|e| !e.is_empty()) {
            let tag = self.etiqueta(nome)?;
            let posicao: i64 = self
                .conexao
                .query_row("SELECT IFNULL(MAX(Position), -1) + 1 FROM TagMap WHERE TagId=?", [tag], |r| r.get(0))
                .map_err(sql)?;
            self.conexao
                .execute("INSERT OR IGNORE INTO TagMap (NoteId, TagId, Position) VALUES (?, ?, ?)", params![nota, tag, posicao])
                .map_err(sql)?;
        }
        Ok(())
    }

    /// O `TagId` de uma etiqueta (Type 1), criada se faltar.
    pub fn etiqueta(&self, nome: &str) -> Resultado<i64> {
        if let Some(t) = self.conexao.query_row("SELECT TagId FROM Tag WHERE Type=1 AND Name=?", [nome], |r| r.get(0)).optional().map_err(sql)? {
            return Ok(t);
        }
        self.conexao.execute("INSERT INTO Tag (Type, Name) VALUES (1, ?)", [nome]).map_err(sql)?;
        Ok(self.conexao.last_insert_rowid())
    }

    /// As etiquetas (Type 1) e quantas notas cada uma tem.
    pub fn etiquetas(&self) -> Resultado<Vec<(String, i64)>> {
        let mut c = self
            .conexao
            .prepare("SELECT t.Name, COUNT(m.NoteId) FROM Tag t LEFT JOIN TagMap m ON m.TagId=t.TagId WHERE t.Type=1 GROUP BY t.TagId ORDER BY t.Name COLLATE NOCASE")
            .map_err(sql)?;
        let linhas = c.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(sql)?;
        linhas.collect::<Result<_, _>>().map_err(sql)
    }

    pub fn apagar_etiqueta(&self, nome: &str) -> Resultado<()> {
        self.conexao.execute("DELETE FROM TagMap WHERE TagId IN (SELECT TagId FROM Tag WHERE Type=1 AND Name=?)", [nome]).map_err(sql)?;
        self.conexao.execute("DELETE FROM Tag WHERE Type=1 AND Name=?", [nome]).map_err(sql)?;
        Ok(())
    }

    pub fn apagar_nota(&self, guid: &str) -> Resultado<()> {
        self.conexao.execute("DELETE FROM TagMap WHERE NoteId IN (SELECT NoteId FROM Note WHERE Guid=?)", [guid]).map_err(sql)?;
        self.conexao.execute("DELETE FROM Note WHERE Guid=?", [guid]).map_err(sql)?;
        Ok(())
    }

    fn etiquetas_da_nota(&self, nota: i64) -> Vec<String> {
        let Ok(mut c) = self.conexao.prepare("SELECT t.Name FROM TagMap m JOIN Tag t ON t.TagId=m.TagId WHERE m.NoteId=? ORDER BY t.Name") else { return vec![] };
        c.query_map([nota], |r| r.get(0)).map(|l| l.filter_map(Result::ok).collect()).unwrap_or_default()
    }

    fn nota_da_linha(&self, r: &rusqlite::Row) -> rusqlite::Result<(i64, Nota)> {
        let id: i64 = r.get(0)?;
        Ok((
            id,
            Nota {
                guid: r.get(1)?,
                titulo: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                texto: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                bloco: r.get(4)?,
                grifo: r.get(5)?,
                cor: r.get::<_, Option<i64>>(6)?.unwrap_or(0),
                etiquetas: vec![],
                alterada: r.get(7)?,
            },
        ))
    }

    /// Grifos, notas e campos de resposta de um documento ou capítulo.
    pub fn do_lugar(&self, id: &Identidade) -> Resultado<DoLugar> {
        let mut saida = DoLugar::default();
        let Some(loc) = self.local_existente(id)? else { return Ok(saida) };
        let mut c = self
            .conexao
            .prepare(
                "SELECT u.UserMarkGuid, u.ColorIndex, b.Identifier, b.StartToken, b.EndToken FROM UserMark u \
                 JOIN BlockRange b ON b.UserMarkId=u.UserMarkId WHERE u.LocationId=? ORDER BY u.UserMarkId, b.Identifier",
            )
            .map_err(sql)?;
        let linhas: Vec<(String, i64, i64, Option<i64>, Option<i64>)> =
            c.query_map([loc], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))).map_err(sql)?.filter_map(Result::ok).collect();
        for (guid, cor, bloco, ini, fim) in linhas {
            let faixa = Faixa { bloco, inicio: ini.unwrap_or(0), fim: fim.unwrap_or(ini.unwrap_or(0)) };
            match saida.grifos.last_mut() {
                Some(g) if g.guid == guid => g.faixas.push(faixa),
                _ => saida.grifos.push(Grifo { guid, cor, faixas: vec![faixa] }),
            }
        }
        let mut c = self
            .conexao
            .prepare(
                "SELECT n.NoteId, n.Guid, n.Title, n.Content, n.BlockIdentifier, u.UserMarkGuid, u.ColorIndex, n.LastModified \
                 FROM Note n LEFT JOIN UserMark u ON u.UserMarkId=n.UserMarkId WHERE n.LocationId=? ORDER BY n.BlockIdentifier, n.NoteId",
            )
            .map_err(sql)?;
        let notas: Vec<(i64, Nota)> = c.query_map([loc], |r| self.nota_da_linha(r)).map_err(sql)?.filter_map(Result::ok).collect();
        saida.notas = notas.into_iter().map(|(n, mut nota)| { nota.etiquetas = self.etiquetas_da_nota(n); nota }).collect();
        let mut c = self.conexao.prepare("SELECT TextTag, Value FROM InputField WHERE LocationId=?").map_err(sql)?;
        saida.campos = c.query_map([loc], |r| Ok((r.get(0)?, r.get(1)?))).map_err(sql)?.filter_map(Result::ok).collect();
        Ok(saida)
    }

    /// Guarda o que se digitou num campo de resposta (`InputField`), ou apaga se vazio.
    pub fn campo(&self, id: &Identidade, tag: &str, valor: &str) -> Resultado<()> {
        let loc = self.local(id)?;
        if valor.trim().is_empty() {
            self.conexao.execute("DELETE FROM InputField WHERE LocationId=? AND TextTag=?", params![loc, tag]).map_err(sql)?;
        } else {
            self.conexao
                .execute("INSERT OR REPLACE INTO InputField (LocationId, TextTag, Value) VALUES (?, ?, ?)", params![loc, tag, valor])
                .map_err(sql)?;
        }
        Ok(())
    }

    /// Todas as notas, mais novas primeiro; com `filtro`, só as que têm o texto
    /// (título, conteúdo ou etiqueta); com `etiqueta`, só as dela.
    pub fn todas_as_notas(&self, filtro: &str, etiqueta: Option<&str>) -> Resultado<Vec<NotaComOrigem>> {
        let mut c = self
            .conexao
            .prepare(
                "SELECT n.NoteId, n.Guid, n.Title, n.Content, n.BlockIdentifier, u.UserMarkGuid, u.ColorIndex, n.LastModified, \
                        IFNULL(l.Title, ''), IFNULL(l.KeySymbol, ''), l.DocumentId, l.BookNumber, l.ChapterNumber, l.IssueTagNumber \
                   FROM Note n LEFT JOIN UserMark u ON u.UserMarkId=n.UserMarkId LEFT JOIN Location l ON l.LocationId=n.LocationId \
                  ORDER BY n.LastModified DESC, n.NoteId DESC",
            )
            .map_err(sql)?;
        let filtro = filtro.trim().to_lowercase();
        let linhas = c
            .query_map([], |r| {
                let (id, nota) = self.nota_da_linha(r)?;
                Ok((id, nota, r.get::<_, String>(8)?, r.get::<_, String>(9)?, r.get(10)?, r.get(11)?, r.get(12)?, r.get::<_, Option<i64>>(13)?))
            })
            .map_err(sql)?;
        let mut saida = vec![];
        for (id, mut nota, origem, simbolo, meps, livro, capitulo, edicao) in linhas.filter_map(Result::ok) {
            nota.etiquetas = self.etiquetas_da_nota(id);
            if let Some(e) = etiqueta {
                if !nota.etiquetas.iter().any(|x| x == e) {
                    continue;
                }
            }
            if !filtro.is_empty() {
                let tudo = format!("{} {} {} {}", nota.titulo, nota.texto, nota.etiquetas.join(" "), origem).to_lowercase();
                if !tudo.contains(&filtro) {
                    continue;
                }
            }
            saida.push(NotaComOrigem { nota, origem, simbolo, meps, livro, capitulo, edicao: edicao.unwrap_or(0) });
        }
        Ok(saida)
    }

    /// Os marcadores da publicação (10 vagas).
    pub fn marcadores(&self, id: &Identidade) -> Resultado<Vec<Marcador>> {
        let Some(publ): Option<i64> = self
            .conexao
            .query_row(
                "SELECT LocationId FROM Location WHERE Type=1 AND KeySymbol=? AND IssueTagNumber=? AND MepsLanguage=?",
                params![id.simbolo, id.edicao, id.lingua],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?
        else {
            return Ok(vec![]);
        };
        let mut c = self
            .conexao
            .prepare(
                "SELECT b.Slot, b.Title, IFNULL(b.Snippet, ''), l.DocumentId, l.BookNumber, l.ChapterNumber, b.BlockIdentifier \
                 FROM Bookmark b JOIN Location l ON l.LocationId=b.LocationId WHERE b.PublicationLocationId=? ORDER BY b.Slot",
            )
            .map_err(sql)?;
        let linhas = c
            .query_map([publ], |r| {
                Ok(Marcador { slot: r.get(0)?, titulo: r.get(1)?, trecho: r.get(2)?, meps: r.get(3)?, livro: r.get(4)?, capitulo: r.get(5)?, bloco: r.get(6)? })
            })
            .map_err(sql)?;
        linhas.collect::<Result<_, _>>().map_err(sql)
    }

    /// Põe (ou troca) o marcador da vaga `slot` neste lugar.
    pub fn marcar(&self, id: &Identidade, slot: i64, bloco: Option<i64>, trecho: &str) -> Resultado<()> {
        if !(0..10).contains(&slot) {
            return Err("vaga inválida".into());
        }
        let loc = self.local(id)?;
        let publ = self.local_da_publicacao(id)?;
        self.conexao.execute("DELETE FROM Bookmark WHERE PublicationLocationId=? AND Slot=?", params![publ, slot]).map_err(sql)?;
        let (tipo, ident) = match bloco {
            Some(b) => (id.bloco(), Some(b)),
            None => (0, None),
        };
        self.conexao
            .execute(
                "INSERT INTO Bookmark (LocationId, PublicationLocationId, Slot, Title, Snippet, BlockType, BlockIdentifier) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![loc, publ, slot, id.titulo, trecho, tipo, ident],
            )
            .map_err(sql)?;
        Ok(())
    }

    pub fn desmarcar(&self, id: &Identidade, slot: i64) -> Resultado<()> {
        let publ = self.local_da_publicacao(id)?;
        self.conexao.execute("DELETE FROM Bookmark WHERE PublicationLocationId=? AND Slot=?", params![publ, slot]).map_err(sql)?;
        Ok(())
    }
}

/// Um `userData.db` vazio, igual ao que o JW Library cria.
pub fn criar_vazio(conexao: &Connection) -> Resultado<()> {
    // A linha de LastModified entra antes dos gatilhos que proíbem mexer nela.
    let (tabelas, gatilhos) = ESQUEMA.split_at(ESQUEMA.find("CREATE TRIGGER").unwrap_or(ESQUEMA.len()));
    conexao.execute_batch(tabelas).map_err(sql)?;
    conexao
        .execute_batch(&format!(
            "INSERT INTO grdb_migrations (identifier) VALUES ('v14');
             INSERT INTO android_metadata (locale) VALUES ('pt_BR');
             INSERT INTO PlaylistItemAccuracy (PlaylistItemAccuracyId, Description) VALUES (1, 'Accurate'), (2, 'NeedsUserVerification');
             INSERT INTO LastModified (LastModified) VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
             PRAGMA user_version = {VERSAO_DO_ESQUEMA};"
        ))
        .map_err(sql)?;
    conexao.execute_batch(gatilhos).map_err(sql)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn doc() -> Identidade {
        Identidade { meps: Some(2026682), livro: None, capitulo: None, simbolo: "w".into(), edicao: 20261200, lingua: 5, titulo: "Artigo".into() }
    }

    #[test]
    fn grifo_nota_etiqueta_e_marcador() {
        let dir = std::env::temp_dir().join(format!("pessoal-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let p = Pessoal::abrir(&dir).unwrap();
        let id = doc();
        let g = p.grifar(&id, &[Faixa { bloco: 5, inicio: 2, fim: 7 }], 3, None).unwrap();
        assert_eq!(g, g.to_uppercase());
        let n = p.anotar(&id, None, Some(5), Some(&g), "Título", "Texto", &["Oração".into()]).unwrap();
        let lugar = p.do_lugar(&id).unwrap();
        assert_eq!(lugar.grifos.len(), 1);
        assert_eq!(lugar.grifos[0].faixas, vec![Faixa { bloco: 5, inicio: 2, fim: 7 }]);
        assert_eq!(lugar.notas[0].cor, 3);
        assert_eq!(lugar.notas[0].etiquetas, vec!["Oração".to_string()]);
        // Recolorir mantém o mesmo grifo.
        p.grifar(&id, &[Faixa { bloco: 5, inicio: 2, fim: 9 }], 1, Some(g.clone())).unwrap();
        assert_eq!(p.do_lugar(&id).unwrap().grifos.len(), 1);
        // Tirar o grifo deixa a nota.
        p.remover_grifo(&g).unwrap();
        let lugar = p.do_lugar(&id).unwrap();
        assert!(lugar.grifos.is_empty());
        assert_eq!(lugar.notas.len(), 1);
        assert_eq!(p.todas_as_notas("oração", None).unwrap().len(), 1);
        assert_eq!(p.todas_as_notas("", Some("Oração")).unwrap().len(), 1);
        p.marcar(&id, 0, Some(5), "trecho").unwrap();
        assert_eq!(p.marcadores(&id).unwrap().len(), 1);
        p.apagar_nota(&n).unwrap();
        assert!(p.do_lugar(&id).unwrap().notas.is_empty());
        // As regras (CHECK, FOREIGN KEY) do esquema do JW Library aceitaram tudo.
        let erros: i64 = p.conexao.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
        assert_eq!(erros, 0);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn biblia_usa_versiculo() {
        let dir = std::env::temp_dir().join(format!("pessoal-b-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let p = Pessoal::abrir(&dir).unwrap();
        let id = Identidade { meps: None, livro: Some(43), capitulo: Some(3), simbolo: "nwtsty".into(), edicao: 0, lingua: 5, titulo: "João 3".into() };
        p.grifar(&id, &[Faixa { bloco: 16, inicio: 0, fim: 4 }], 1, None).unwrap();
        let tipo: i64 = p.conexao.query_row("SELECT BlockType FROM BlockRange", [], |r| r.get(0)).unwrap();
        assert_eq!(tipo, 2);
        let _ = std::fs::remove_file(&dir);
    }
}
