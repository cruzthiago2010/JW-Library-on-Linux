//! O catálogo completo de publicações do jw.org (o mesmo que o JW Library usa:
//! `app.jw-cdn.org/catalogs/publications/v4`). O arquivo oficial tem todas as
//! línguas (~60 MB compactado, ~220 MB aberto); aqui ele é baixado de vez em
//! quando, só as línguas da lista `IDIOMAS` são copiadas para um `catalogo.db`
//! pequeno, e o resto é apagado.
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::io::Seek;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags};
use serde::Serialize;

const MANIFESTO: &str = "https://app.jw-cdn.org/catalogs/publications/v4/manifest.json";
const BASE: &str = "https://app.jw-cdn.org/catalogs/publications/v4/";
/// As capas: `IMAGENS` + `NameFragment` do catálogo.
pub const IMAGENS: &str = "https://app.jw-cdn.org/catalogs/publications/";

/// Código do jw.org → `MepsLanguageId` (o mesmo número das marcas do JW Library).
pub const LINGUAS: &[(&str, i64)] = &[("T", 5), ("TPO", 785), ("E", 0), ("S", 1), ("F", 3), ("X", 2), ("I", 4)];

/// O número MEPS de um código: os da lista fixa e os descobertos (`linguas.json`).
pub fn meps_de_em(dados: &Path, codigo: &str) -> Option<i64> {
    LINGUAS.iter().find(|(c, _)| *c == codigo).map(|(_, m)| *m).or_else(|| descobertas(dados).get(codigo).copied())
}

pub fn meps_de(codigo: &str) -> Option<i64> {
    LINGUAS.iter().find(|(c, _)| *c == codigo).map(|(_, m)| *m)
}

fn arquivo_linguas(dados: &Path) -> PathBuf {
    dados.join("linguas.json")
}

/// Código → MEPS das línguas que o usuário escolheu além das fixas.
pub fn descobertas(dados: &Path) -> std::collections::BTreeMap<String, i64> {
    std::fs::read_to_string(arquivo_linguas(dados)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Todos os MEPS que entram no catálogo pequeno.
fn todos_os_meps(dados: &Path) -> Vec<i64> {
    let mut v: Vec<i64> = LINGUAS.iter().map(|(_, m)| *m).chain(descobertas(dados).into_values()).collect();
    v.sort();
    v.dedup();
    v
}

/// Publicações pequenas que existem em quase todas as línguas (folhetos e brochuras).
const AMOSTRAS: &[&str] = &["T-31", "T-30", "T-32", "T-33", "T-34", "T-35", "T-36", "T-37", "fg", "ll", "lff"];

/// Descobre o número MEPS de um código baixando uma publicação pequena nessa
/// língua e lendo o `manifest.json` (`publication.language`). Guarda em `linguas.json`.
pub fn descobrir(dados: &Path, codigo: &str, cliente: &mut crate::Cliente) -> Result<i64, String> {
    if let Some(m) = meps_de_em(dados, codigo) {
        return Ok(m);
    }
    if !codigo.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || codigo.is_empty() || codigo.len() > 12 {
        return Err("código de língua inválido".into());
    }
    let partes = dados.join("partes");
    for sigla in AMOSTRAS {
        let Ok(Some(arquivo)) = cliente.consultar(&crate::Pedido::novo(sigla, codigo, "")) else { continue };
        if arquivo.tamanho > 40 * 1024 * 1024 {
            continue;
        }
        let Ok(jwpub) = cliente.baixar(&arquivo, &partes, &mut |_, _| {}) else { continue };
        let meps = (|| -> Option<i64> {
            let f = std::fs::File::open(&jwpub).ok()?;
            let mut z = zip::ZipArchive::new(f).ok()?;
            let mut texto = String::new();
            z.by_name("manifest.json").ok()?.read_to_string(&mut texto).ok()?;
            let v: serde_json::Value = serde_json::from_str(&texto).ok()?;
            v.pointer("/publication/language")?.as_i64()
        })();
        let _ = std::fs::remove_file(&jwpub);
        if let Some(m) = meps {
            let mut todas = descobertas(dados);
            todas.insert(codigo.to_string(), m);
            std::fs::create_dir_all(dados).map_err(|e| e.to_string())?;
            std::fs::write(arquivo_linguas(dados), serde_json::to_string_pretty(&todas).unwrap()).map_err(|e| e.to_string())?;
            return Ok(m);
        }
    }
    Err(format!("{codigo}: não há publicações nesse idioma"))
}

/// Refaz o catálogo pequeno a partir do completo guardado (depois de uma língua nova).
pub fn refazer(dados: &Path) -> Result<(), String> {
    let gz = dados.join("catalogo.db.gz");
    let versao = versao_local(dados).map(|(v, _)| v).unwrap_or_default();
    if !gz.is_file() {
        return Err("catálogo completo não encontrado".into());
    }
    let grande = dados.join("catalogo-completo.db");
    let abrir = || -> Result<(), String> {
        let mut entrada = flate2::read::GzDecoder::new(std::fs::File::open(&gz).map_err(|e| e.to_string())?);
        let mut saida = std::fs::File::create(&grande).map_err(|e| e.to_string())?;
        std::io::copy(&mut entrada, &mut saida).map_err(|e| e.to_string())?;
        reduzir(&grande, &caminho(dados), &versao, &todos_os_meps(dados))
    };
    let r = abrir();
    let _ = std::fs::remove_file(&grande);
    r
}

/// `PublicationTypeId` → a categoria da Biblioteca (as mesmas chaves de
/// `leitor_nucleo::biblioteca::CATEGORIAS`).
pub fn categoria_do_tipo(tipo: i64) -> &'static str {
    match tipo {
        1 => "biblia",
        2 => "livros",
        4 => "brochuras",
        10 => "folhetos",
        22 => "series",
        14 => "sentinela",
        13 => "despertai",
        30 => "apostilas",
        7 => "ministerio",
        31 => "programas",
        6 | 17 => "orientacoes",
        _ => "outras",
    }
}

/// A ordem da tela PUBLICAÇÕES do JW Library.
pub const ORDEM: &[&str] =
    &["biblia", "livros", "brochuras", "folhetos", "series", "sentinela", "despertai", "apostilas", "ministerio", "programas", "orientacoes", "outras"];

pub fn caminho(dados: &Path) -> PathBuf {
    dados.join("catalogo.db")
}

fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent(crate::api::AGENTE)
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(120))
        .build()
}

/// A versão do catálogo que está aqui (o `current` do manifesto) e quando foi baixado.
pub fn versao_local(dados: &Path) -> Option<(String, u64)> {
    let c = Connection::open_with_flags(caminho(dados), OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    c.query_row("SELECT versao, baixado_em FROM Estado", [], |l| Ok((l.get(0)?, l.get::<_, i64>(1)? as u64))).ok()
}

/// Baixa o catálogo se o do servidor mudou. Devolve se trocou.
pub fn atualizar(dados: &Path, progresso: &mut dyn FnMut(u64, u64)) -> Result<bool, String> {
    let ag = agente();
    let manifesto: serde_json::Value = ag
        .get(MANIFESTO)
        .call()
        .map_err(|e| format!("catálogo do jw.org: {e}"))?
        .into_string()
        .map_err(|e| format!("catálogo do jw.org: {e}"))
        .and_then(|t| serde_json::from_str(&t).map_err(|e| format!("catálogo do jw.org: {e}")))?;
    let atual = manifesto.get("current").and_then(|v| v.as_str()).ok_or("catálogo do jw.org sem versão")?.to_string();
    if !atual.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("versão do catálogo inválida".into());
    }
    if versao_local(dados).map(|(v, _)| v) == Some(atual.clone()) {
        return Ok(false);
    }
    std::fs::create_dir_all(dados).map_err(|e| e.to_string())?;
    let gz = dados.join("catalogo.db.gz");
    let parcial = dados.join("catalogo.db.gz.part");
    let resposta = ag.get(&format!("{BASE}{atual}/catalog.db.gz")).call().map_err(|e| format!("baixar o catálogo: {e}"))?;
    let total: u64 = resposta.header("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    {
        let mut leitor = resposta.into_reader();
        let mut saida = std::fs::File::create(&parcial).map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; 256 * 1024];
        let mut lido = 0u64;
        loop {
            let n = leitor.read(&mut buf).map_err(|e| format!("catálogo interrompido: {e}"))?;
            if n == 0 {
                break;
            }
            saida.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            lido += n as u64;
            progresso(lido, total);
        }
    }
    std::fs::rename(&parcial, &gz).map_err(|e| e.to_string())?;
    let grande = dados.join("catalogo-completo.db");
    {
        let mut entrada = flate2::read::GzDecoder::new(std::fs::File::open(&gz).map_err(|e| e.to_string())?);
        let mut saida = std::fs::File::create(&grande).map_err(|e| e.to_string())?;
        std::io::copy(&mut entrada, &mut saida).map_err(|e| format!("catálogo: {e}"))?;
    }
    let resultado = reduzir(&grande, &caminho(dados), &atual, &todos_os_meps(dados));
    let _ = std::fs::remove_file(&grande);
    resultado.map(|_| true)
}

/// Copia só as línguas de `LINGUAS` do catálogo completo para o pequeno.
fn reduzir(grande: &Path, destino: &Path, versao: &str, meps: &[i64]) -> Result<(), String> {
    let novo = destino.with_extension("db.novo");
    let _ = std::fs::remove_file(&novo);
    let c = Connection::open(&novo).map_err(|e| e.to_string())?;
    let erro = |e: rusqlite::Error| format!("catálogo: {e}");
    c.execute("ATTACH DATABASE ?1 AS jw", [grande.to_string_lossy()]).map_err(erro)?;
    let linguas = meps.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(",");
    c.execute_batch(&format!(
        "CREATE TABLE Pub (lingua INTEGER, simbolo TEXT, edicao INTEGER, titulo TEXT, titulo_edicao TEXT, curto TEXT,
             ano INTEGER, tipo INTEGER, tamanho INTEGER, modificado TEXT, capa TEXT, larga TEXT);
         INSERT INTO Pub
           SELECT p.MepsLanguageId, p.KeySymbol, p.IssueTagNumber, p.Title, p.IssueTitle, p.ShortTitle, p.Year,
                  p.PublicationTypeId, a.Size, a.LastModified,
                  (SELECT i.NameFragment FROM jw.PublicationAssetImageMap m JOIN jw.ImageAsset i ON i.Id = m.ImageAssetId
                     WHERE m.PublicationAssetId = a.Id AND i.Width = i.Height ORDER BY abs(i.Width - 270) LIMIT 1),
                  (SELECT i.NameFragment FROM jw.PublicationAssetImageMap m JOIN jw.ImageAsset i ON i.Id = m.ImageAssetId
                     WHERE m.PublicationAssetId = a.Id AND i.Width = 2 * i.Height ORDER BY abs(i.Width - 560) LIMIT 1)
           FROM jw.Publication p JOIN jw.PublicationAsset a ON a.PublicationId = p.Id
           WHERE p.MepsLanguageId IN ({linguas}) AND p.KeySymbol IS NOT NULL;
         CREATE INDEX PubLingua ON Pub (lingua, tipo, ano);
         CREATE TABLE Estado (versao TEXT, baixado_em INTEGER);"
    ))
    .map_err(erro)?;
    c.execute("INSERT INTO Estado VALUES (?1, ?2)", params![versao, crate::agora() as i64]).map_err(erro)?;
    c.execute("DETACH DATABASE jw", []).map_err(erro)?;
    drop(c);
    std::fs::rename(&novo, destino).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Clone)]
pub struct PubDoCatalogo {
    pub simbolo: String,
    pub edicao: i64,
    pub titulo: String,
    pub titulo_edicao: Option<String>,
    pub ano: i64,
    pub tamanho: i64,
    pub modificado: String,
    pub capa: Option<String>,
    pub larga: Option<String>,
    pub categoria: &'static str,
}

fn abrir(dados: &Path) -> Result<Connection, String> {
    Connection::open_with_flags(caminho(dados), OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|_| "o catálogo ainda não foi baixado".to_string())
}

/// Quantas publicações cada categoria tem numa língua, na ordem do JW Library.
pub fn categorias(dados: &Path, lingua: i64) -> Result<Vec<(&'static str, i64)>, String> {
    let c = abrir(dados)?;
    let mut s = c.prepare("SELECT tipo, count(*) FROM Pub WHERE lingua = ?1 GROUP BY tipo").map_err(|e| e.to_string())?;
    let mut contas: std::collections::BTreeMap<&'static str, i64> = Default::default();
    for linha in s.query_map([lingua], |l| Ok((l.get::<_, i64>(0)?, l.get::<_, i64>(1)?))).map_err(|e| e.to_string())?.flatten() {
        *contas.entry(categoria_do_tipo(linha.0)).or_default() += linha.1;
    }
    Ok(ORDEM.iter().filter_map(|k| contas.get(k).map(|n| (*k, *n))).collect())
}

/// As publicações de uma categoria (de um ano, nos periódicos), as mais novas primeiro.
pub fn lista(dados: &Path, lingua: i64, categoria: &str, ano: Option<i64>) -> Result<Vec<PubDoCatalogo>, String> {
    let c = abrir(dados)?;
    let mut s = c
        .prepare(
            "SELECT simbolo, edicao, titulo, titulo_edicao, ano, tamanho, modificado, capa, larga, tipo FROM Pub
             WHERE lingua = ?1 AND (?2 IS NULL OR ano = ?2) ORDER BY ano DESC, edicao DESC, titulo",
        )
        .map_err(|e| e.to_string())?;
    let linhas = s
        .query_map(params![lingua, ano], |l| {
            Ok(PubDoCatalogo {
                simbolo: l.get(0)?,
                edicao: l.get(1)?,
                titulo: l.get(2)?,
                titulo_edicao: l.get(3)?,
                ano: l.get(4)?,
                tamanho: l.get(5)?,
                modificado: l.get(6)?,
                capa: l.get(7)?,
                larga: l.get(8)?,
                categoria: categoria_do_tipo(l.get(9)?),
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(linhas.flatten().filter(|p| p.categoria == categoria).collect())
}

/// Os anos de uma categoria, do mais novo ao mais velho.
pub fn anos(dados: &Path, lingua: i64, categoria: &str) -> Result<Vec<i64>, String> {
    let mut v: Vec<i64> = lista(dados, lingua, categoria, None)?.iter().map(|p| p.ano).collect();
    v.dedup();
    Ok(v)
}

/// O texto do `issue` da API de download: 20261100 → "202611"; 19930122 → "19930122".
pub fn edicao_da_api(edicao: i64) -> String {
    match edicao {
        0 => String::new(),
        e if e % 100 == 0 => (e / 100).to_string(),
        e => e.to_string(),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn edicao_e_categoria() {
        assert_eq!(edicao_da_api(20261100), "202611");
        assert_eq!(edicao_da_api(19930122), "19930122");
        assert_eq!(edicao_da_api(0), "");
        assert_eq!(categoria_do_tipo(14), "sentinela");
        assert_eq!(meps_de("TPO"), Some(785));
    }

    #[test]
    fn reduz_o_catalogo_real() {
        let Ok(grande) = std::env::var("CATALOGO_JW") else { return };
        let pasta = std::env::temp_dir().join(format!("catalogo-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        reduzir(Path::new(&grande), &caminho(&pasta), "teste", &[5, 0]).unwrap();
        let cats = categorias(&pasta, 5).unwrap();
        println!("{cats:?}");
        assert_eq!(cats[0].0, "biblia");
        let w = lista(&pasta, 5, "sentinela", Some(2026)).unwrap();
        assert!(w.iter().any(|p| p.simbolo == "w" && p.edicao == 20261100));
        println!("{:?}", &w[0]);
        println!("{} KB", std::fs::metadata(caminho(&pasta)).unwrap().len() / 1024);
        let _ = std::fs::remove_dir_all(pasta);
    }
}
