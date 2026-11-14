//! Leitura do `.jwpub` — o mesmo arquivo que o JW Library abre.
//!
//! Porte do `app/library/jwpub.py` do servidor: o `.jwpub` é um ZIP com
//! `manifest.json` e `contents`, e `contents` é outro ZIP com o banco SQLite e
//! as imagens. O campo `Content` de cada documento vem comprimido (zlib) e
//! cifrado (AES-128-CBC); a chave sai do SHA-256 de
//! `"<língua>_<símbolo>_<ano>[_<edição>]"` combinado por XOR com uma constante
//! (16 primeiros bytes = chave, 16 últimos = vetor inicial).
//!
//! O arquivo é **instalado** uma vez: o banco e as imagens vão para uma pasta
//! (`<raiz>/<nome>/publicacao.db` + `midia/`), e daí em diante é só leitura.

use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use rusqlite::{types::Value, Connection, OpenFlags, OptionalExtension};
use sha2::{Digest, Sha256};

const CONSTANTE: [u8; 32] = [
    0x11, 0xcb, 0xb5, 0x58, 0x7e, 0x32, 0x84, 0x6d, 0x4c, 0x26, 0x79, 0x0c, 0x63, 0x3d, 0xa2, 0x89,
    0xf6, 0x6f, 0xe5, 0x84, 0x2a, 0x3a, 0x58, 0x5c, 0xe1, 0xbc, 0x3a, 0x29, 0x4a, 0xf5, 0xad, 0xa7,
];
const SQLITE: &[u8] = b"SQLite format 3\0";
pub const BANCO: &str = "publicacao.db";
pub const MIDIA: &str = "midia";

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub type Resultado<T> = Result<T, String>;

fn erro<E: std::fmt::Display>(contexto: &str) -> impl Fn(E) -> String + '_ {
    move |e| format!("{contexto}: {e}")
}

/// A chave e o vetor inicial de uma publicação.
pub fn chave_e_vetor(lingua: i64, simbolo: &str, ano: i64, edicao: i64) -> ([u8; 16], [u8; 16]) {
    let mut rotulo = format!("{lingua}_{simbolo}_{ano}");
    if edicao != 0 {
        rotulo.push_str(&format!("_{edicao}"));
    }
    let resumo = Sha256::digest(rotulo.as_bytes());
    let mut chave = [0u8; 16];
    let mut vetor = [0u8; 16];
    for i in 0..16 {
        chave[i] = resumo[i] ^ CONSTANTE[i];
        vetor[i] = resumo[16 + i] ^ CONSTANTE[16 + i];
    }
    (chave, vetor)
}

/// O HTML de um campo `Content`. O enchimento do bloco sobra no fim; o zlib
/// para sozinho onde o texto acaba.
pub fn decifrar(bruto: &[u8], chave: &[u8; 16], vetor: &[u8; 16]) -> Resultado<String> {
    let mut dados = bruto.to_vec();
    let corte = dados.len() - dados.len() % 16;
    let claro = Aes128CbcDec::new(chave.into(), vetor.into())
        .decrypt_padded_mut::<NoPadding>(&mut dados[..corte])
        .map_err(erro("decifrar"))?;
    let mut texto = Vec::new();
    let mut leitor = flate2::read::ZlibDecoder::new(claro);
    leitor.read_to_end(&mut texto).map_err(erro("descomprimir"))?;
    Ok(String::from_utf8_lossy(&texto).into_owned())
}

/// Só o nome do arquivo, sem pasta — nada de `..` escrevendo fora do lugar.
pub fn nome_seguro(nome: &str) -> Option<String> {
    let base = nome.replace('\\', "/");
    let base = base.rsplit('/').next().unwrap_or("");
    if base.is_empty() || base == "." || base == ".." {
        None
    } else {
        Some(base.to_string())
    }
}

/// Instala o `.jwpub` em `<raiz>/<nome sem extensão>/`. Não repete se o
/// tamanho do arquivo é o mesmo da última vez. Devolve o nome da pasta.
pub fn instalar(arquivo: &Path, raiz: &Path) -> Resultado<String> {
    let nome = arquivo
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(nome_seguro)
        .ok_or("nome de arquivo inválido")?;
    let pasta = raiz.join(&nome);
    let marca = pasta.join("instalado.json");
    let tamanho = fs::metadata(arquivo).map_err(erro("ler"))?.len();
    if let Ok(texto) = fs::read_to_string(&marca) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&texto) {
            if json.get("tamanho").and_then(|t| t.as_u64()) == Some(tamanho) {
                return Ok(nome);
            }
        }
    }

    let externo = fs::File::open(arquivo).map_err(erro("abrir"))?;
    let mut externo = zip::ZipArchive::new(externo).map_err(erro("zip externo"))?;
    let mut manifesto = String::new();
    externo
        .by_name("manifest.json")
        .map_err(erro("manifest.json"))?
        .read_to_string(&mut manifesto)
        .map_err(erro("manifest.json"))?;
    let manifesto: serde_json::Value = serde_json::from_str(&manifesto).map_err(erro("manifest.json"))?;
    let nome_do_banco = manifesto
        .pointer("/publication/fileName")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let mut conteudo = Vec::new();
    externo
        .by_name("contents")
        .map_err(erro("contents"))?
        .read_to_end(&mut conteudo)
        .map_err(erro("contents"))?;
    let mut interno = zip::ZipArchive::new(Cursor::new(conteudo)).map_err(erro("zip interno"))?;

    fs::create_dir_all(pasta.join(MIDIA)).map_err(erro("criar pasta"))?;
    let mut achou = false;
    for i in 0..interno.len() {
        let mut item = interno.by_index(i).map_err(erro("item do zip"))?;
        if item.is_dir() {
            continue;
        }
        let nome_item = item.name().to_string();
        let mut dados = Vec::with_capacity(item.size() as usize);
        item.read_to_end(&mut dados).map_err(erro("ler item"))?;
        let e_o_banco = dados.starts_with(SQLITE)
            && (Some(&nome_item) == nome_do_banco.as_ref() || !achou);
        if e_o_banco {
            fs::write(pasta.join(BANCO), &dados).map_err(erro("gravar banco"))?;
            achou = true;
        } else if let Some(base) = nome_seguro(&nome_item) {
            fs::write(pasta.join(MIDIA).join(base), &dados).map_err(erro("gravar mídia"))?;
        }
    }
    if !achou {
        return Err(format!("{nome}: sem banco de dados"));
    }
    fs::write(&marca, serde_json::json!({ "tamanho": tamanho }).to_string())
        .map_err(erro("gravar marca"))?;
    Ok(nome)
}

fn inteiro(valor: Value) -> i64 {
    match valor {
        Value::Integer(n) => n,
        Value::Text(t) => t.trim().parse().unwrap_or(0),
        Value::Real(r) => r as i64,
        _ => 0,
    }
}

fn texto(valor: Value) -> String {
    match valor {
        Value::Text(t) => t,
        Value::Integer(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Um `.jwpub` instalado.
pub struct Publicacao {
    pub pasta: String,
    dir: PathBuf,
    chave: [u8; 16],
    vetor: [u8; 16],
    pub titulo: String,
    pub curto: String,
    pub simbolo: String,
    /// O símbolo sem data ("w", "mwb", "wcg"): o `pub-…` e o `kind` do site.
    pub sigla: String,
    pub ano: i64,
    /// O `IssueTagNumber` (w set/2026 = 20260900; livro = 0): o da `Location`.
    pub edicao: i64,
    /// `PublicationCategorySymbol` ("bi", "bk", "w", "mwb"…) e `PublicationType`
    /// ("Bible", "Book"…): a categoria na Biblioteca do Arquivo.
    pub categoria: String,
    pub tipo: String,
    /// `MepsLanguageIndex` (português = 5): o `MepsLanguage` das marcas do usuário.
    pub lingua: i64,
}

#[derive(serde::Serialize, Clone)]
pub struct ItemDoSumario {
    pub id: i64,
    pub titulo: String,
    pub documento: Option<i64>,
    pub miniatura: Option<String>,
    pub filhos: Vec<ItemDoSumario>,
}

pub struct Documento {
    pub id: i64,
    pub meps: i64,
    pub classe: i64,
    pub titulo: String,
    pub html: String,
}

impl Publicacao {
    pub fn abrir(raiz: &Path, pasta: &str) -> Resultado<Self> {
        let pasta = nome_seguro(pasta).ok_or("pasta inválida")?;
        let dir = raiz.join(&pasta);
        let conexao = Self::conectar(&dir)?;
        let (lingua, simbolo, ano, edicao, titulo, curto, sigla) = conexao
            .query_row(
                "SELECT MepsLanguageIndex, Symbol, Year, IssueTagNumber, Title, ShortTitle, \
                        UndatedSymbol FROM Publication LIMIT 1",
                [],
                |l| {
                    Ok((
                        inteiro(l.get(0)?),
                        texto(l.get(1)?),
                        inteiro(l.get(2)?),
                        inteiro(l.get(3)?),
                        texto(l.get(4)?),
                        texto(l.get(5)?),
                        texto(l.get(6)?),
                    ))
                },
            )
            .map_err(erro("Publication"))?;
        let (chave, vetor) = chave_e_vetor(lingua, &simbolo, ano, edicao);
        let sigla = if sigla.is_empty() { simbolo.clone() } else { sigla };
        // Revista e apostila: o título do número ("A Sentinela, setembro de
        // 2026"), não o do ano ("… — 2026"), como na Biblioteca do JW Library.
        let do_numero: Option<String> = conexao
            .query_row("SELECT Title FROM PublicationIssueProperty LIMIT 1", [], |l| Ok(texto(l.get(0)?)))
            .ok()
            .filter(|t| !t.is_empty());
        let titulo = do_numero.unwrap_or(titulo);
        // Colunas que nem todo .jwpub antigo tem: sem elas, fica vazio.
        let (categoria, tipo) = conexao
            .query_row("SELECT PublicationCategorySymbol, PublicationType FROM Publication LIMIT 1", [], |l| {
                Ok((texto(l.get(0)?), texto(l.get(1)?)))
            })
            .unwrap_or_default();
        Ok(Self { pasta, dir, chave, vetor, titulo, curto, simbolo, sigla, ano, edicao, categoria, tipo, lingua })
    }

    fn conectar(dir: &Path) -> Resultado<Connection> {
        let banco = dir.join(BANCO);
        if !banco.exists() {
            return Err("publicação não instalada".into());
        }
        Connection::open_with_flags(&banco, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(erro("abrir banco"))
    }

    fn banco(&self) -> Resultado<Connection> {
        Self::conectar(&self.dir)
    }

    /// O banco, só leitura, para quem lê tabelas além das de documento (a Bíblia).
    pub fn banco_publico(&self) -> Resultado<Connection> {
        self.banco()
    }

    /// Um campo cifrado qualquer da publicação (capítulo, versículo…) em texto.
    pub fn claro(&self, bruto: &[u8]) -> Resultado<String> {
        decifrar(bruto, &self.chave, &self.vetor)
    }

    /// A capa (categoria 15), ou a primeira miniatura quadrada (9).
    pub fn capa(&self) -> Option<String> {
        let conexao = self.banco().ok()?;
        conexao
            .query_row(
                "SELECT FilePath FROM Multimedia WHERE CategoryType IN (15, 9) \
                 ORDER BY CategoryType DESC, MultimediaId LIMIT 1",
                [],
                |l| l.get::<_, String>(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    /// O sumário como o JW Library mostra (tabela `PublicationViewItem`).
    pub fn sumario(&self) -> Resultado<Vec<ItemDoSumario>> {
        let conexao = self.banco()?;
        let mut consulta = conexao
            .prepare(
                "SELECT i.PublicationViewItemId, i.ParentPublicationViewItemId, i.Title, \
                        i.DefaultDocumentId, \
                        (SELECT m.FilePath FROM DocumentMultimedia dm \
                           JOIN Multimedia m ON m.MultimediaId = dm.MultimediaId \
                          WHERE dm.DocumentId = i.DefaultDocumentId AND m.CategoryType = 9 \
                          LIMIT 1) \
                   FROM PublicationViewItem i ORDER BY i.PublicationViewItemId",
            )
            .map_err(erro("sumário"))?;
        let linhas: Vec<(i64, i64, String, i64, Option<String>)> = consulta
            .query_map([], |l| {
                Ok((
                    l.get(0)?,
                    inteiro(l.get(1)?),
                    texto(l.get(2)?),
                    inteiro(l.get(3)?),
                    l.get(4)?,
                ))
            })
            .map_err(erro("sumário"))?
            .filter_map(Result::ok)
            .collect();

        fn montar(linhas: &[(i64, i64, String, i64, Option<String>)], pai: i64) -> Vec<ItemDoSumario> {
            linhas
                .iter()
                .filter(|l| l.1 == pai)
                .map(|l| ItemDoSumario {
                    id: l.0,
                    titulo: l.2.clone(),
                    documento: (l.3 > 0).then_some(l.3),
                    miniatura: l.4.clone(),
                    filhos: montar(linhas, l.0),
                })
                .collect()
        }
        let mut raiz = montar(&linhas, -1);
        if raiz.is_empty() {
            // Sem sumário no arquivo: a lista dos documentos, em ordem.
            let mut consulta = conexao
                .prepare("SELECT DocumentId, Title FROM Document ORDER BY DocumentId")
                .map_err(erro("documentos"))?;
            raiz = consulta
                .query_map([], |l| {
                    Ok(ItemDoSumario {
                        id: l.get(0)?,
                        titulo: texto(l.get(1)?),
                        documento: Some(l.get(0)?),
                        miniatura: None,
                        filhos: vec![],
                    })
                })
                .map_err(erro("documentos"))?
                .filter_map(Result::ok)
                .collect();
        }
        Ok(raiz)
    }

    pub fn documento(&self, id: i64) -> Resultado<Documento> {
        let conexao = self.banco()?;
        // Documento sem conteúdo existe (os livros da Bíblia de estudo): vazio,
        // como o `_claro` do jwpub.py.
        let (meps, classe, titulo, conteudo): (i64, i64, String, Option<Vec<u8>>) = conexao
            .query_row(
                "SELECT MepsDocumentId, Class, Title, Content FROM Document WHERE DocumentId = ?1",
                [id],
                |l| Ok((inteiro(l.get(0)?), inteiro(l.get(1)?), texto(l.get(2)?), l.get(3)?)),
            )
            .map_err(erro("documento"))?;
        let html = match conteudo {
            Some(bruto) if !bruto.is_empty() => decifrar(&bruto, &self.chave, &self.vetor)?,
            _ => String::new(),
        };
        Ok(Documento { id, meps, classe, titulo, html })
    }

    /// Todos os `DocumentId` do arquivo, em ordem.
    pub fn documentos(&self) -> Resultado<Vec<i64>> {
        let conexao = self.banco()?;
        let mut consulta = conexao
            .prepare("SELECT DocumentId FROM Document ORDER BY DocumentId")
            .map_err(erro("documentos"))?;
        let ids = consulta
            .query_map([], |l| l.get(0))
            .map_err(erro("documentos"))?
            .filter_map(Result::ok)
            .collect();
        Ok(ids)
    }

    /// O `DocumentId` local de um documento pelo número do wol (`MepsDocumentId`).
    pub fn pelo_meps(&self, meps: i64) -> Option<i64> {
        let conexao = self.banco().ok()?;
        conexao
            .query_row("SELECT DocumentId FROM Document WHERE MepsDocumentId = ?1", [meps], |l| l.get(0))
            .optional()
            .ok()
            .flatten()
    }

    pub fn midia(&self, nome: &str) -> Option<PathBuf> {
        let caminho = self.dir.join(MIDIA).join(nome_seguro(nome)?);
        caminho.exists().then_some(caminho)
    }
}

/// As publicações instaladas na raiz, em ordem de nome.
pub fn instaladas(raiz: &Path) -> Vec<Publicacao> {
    let mut nomes: Vec<String> = fs::read_dir(raiz)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.path().join(BANCO).exists())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    nomes.sort();
    nomes.iter().filter_map(|n| Publicacao::abrir(raiz, n).ok()).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn chave_igual_a_do_servidor() {
        // Conferida com o jwpub.py: "5_wcg_2025" (edição 0 não entra no rótulo).
        let (chave, _) = chave_e_vetor(5, "wcg", 2025, 0);
        let (outra, _) = chave_e_vetor(5, "wcg", 2025, 1);
        assert_ne!(chave, outra);
    }

    #[test]
    fn nome_seguro_corta_pastas() {
        assert_eq!(nome_seguro("a/b/../c.jpg").as_deref(), Some("c.jpg"));
        assert_eq!(nome_seguro(".."), None);
        assert_eq!(nome_seguro("x\\y.png").as_deref(), Some("y.png"));
    }
}
