//! Backup e restauração no formato do JW Library (`.jwlibrary`): um ZIP com
//! `manifest.json`, `userData.db` e `default_thumbnail.png`. O `userData.db` é
//! o mesmo banco do estudo pessoal (`pessoal.rs`), então restaurar é trocar o
//! arquivo e criar é empacotá-lo. Regras do manifest conferidas num backup real
//! (versão 1, `schemaVersion` 16, `hash` = SHA-256 do userData.db).
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};

pub type Resultado<T> = Result<T, String>;

const MANIFESTO: &str = "manifest.json";
const BANCO: &str = "userData.db";
const MINIATURA: &str = "default_thumbnail.png";
const IMAGEM: &[u8] = include_bytes!("../../app/icons/32x32.png");
const TABELAS: [&str; 8] = ["Location", "UserMark", "BlockRange", "Note", "Tag", "TagMap", "Bookmark", "LastModified"];
const LIMITE: u64 = 512 * 1024 * 1024;

#[derive(serde::Serialize, Default, Debug)]
pub struct Resumo {
    pub aparelho: String,
    pub data: String,
    pub grifos: i64,
    pub notas: i64,
    pub marcadores: i64,
    pub etiquetas: i64,
}

fn sha256_hex(dados: &[u8]) -> String {
    format!("{:x}", Sha256::digest(dados))
}

fn contar(c: &Connection, tabela: &str, onde: &str) -> i64 {
    c.query_row(&format!("SELECT COUNT(*) FROM {tabela} {onde}"), [], |r| r.get(0)).unwrap_or(0)
}

/// O que há num userData.db.
pub fn resumo(banco: &Path) -> Resultado<Resumo> {
    let c = Connection::open_with_flags(banco, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| e.to_string())?;
    Ok(Resumo {
        grifos: contar(&c, "UserMark", ""),
        notas: contar(&c, "Note", ""),
        marcadores: contar(&c, "Bookmark", ""),
        etiquetas: contar(&c, "Tag", "WHERE Type=1"),
        ..Default::default()
    })
}

/// Confere o `.jwlibrary` e grava o `userData.db` dele em `saida`. Não toca no banco atual.
pub fn extrair(arquivo: &Path, saida: &Path) -> Resultado<Resumo> {
    let f = fs::File::open(arquivo).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(f).map_err(|_| crate::idioma::t("não é um arquivo .jwlibrary (ZIP)", "not a .jwlibrary file (ZIP)").to_string())?;
    let mut total = 0u64;
    for i in 0..zip.len() {
        let e = zip.by_index(i).map_err(|_| crate::idioma::t("o arquivo está corrompido", "the file is corrupted").to_string())?;
        let nome = e.name().to_string();
        if nome.contains('/') || nome.contains('\\') || nome.starts_with('.') {
            return Err(format!("{}: {nome}", crate::idioma::t("o arquivo tem algo que não é de backup do JW Library", "the file contains something that is not part of a JW Library backup")));
        }
        total += e.size();
    }
    if total > LIMITE {
        return Err(crate::idioma::t("o conteúdo do backup é grande demais", "the backup content is too large").into());
    }
    let mut manifesto = String::new();
    zip.by_name(MANIFESTO)
        .map_err(|_| crate::idioma::t("falta o manifest.json no arquivo", "manifest.json is missing from the file").to_string())?
        .take(1024 * 1024)
        .read_to_string(&mut manifesto)
        .map_err(|_| crate::idioma::t("o manifest.json não é válido", "manifest.json is not valid").to_string())?;
    let manifesto: serde_json::Value = serde_json::from_str(&manifesto).map_err(|_| crate::idioma::t("o manifest.json não é válido", "manifest.json is not valid").to_string())?;
    let bloco = manifesto.get("userDataBackup").filter(|v| v.is_object()).ok_or(crate::idioma::t("o arquivo não é um backup do JW Library", "the file is not a JW Library backup"))?;
    let mut dados = vec![];
    zip.by_name(BANCO).map_err(|_| crate::idioma::t("falta o userData.db no arquivo", "userData.db is missing from the file").to_string())?.read_to_end(&mut dados).map_err(|_| crate::idioma::t("o arquivo está corrompido", "the file is corrupted").to_string())?;
    if let Some(h) = bloco.get("hash").and_then(|v| v.as_str()).filter(|h| !h.is_empty()) {
        if h != sha256_hex(&dados) {
            return Err(crate::idioma::t("o userData.db não confere com o manifest (arquivo danificado)", "userData.db does not match the manifest (damaged file)").into());
        }
    }
    if !dados.starts_with(b"SQLite format 3\0") {
        return Err(crate::idioma::t("o userData.db não é um banco do JW Library", "userData.db is not a JW Library database").into());
    }
    fs::write(saida, &dados).map_err(|e| e.to_string())?;
    let conferir = || -> Resultado<()> {
        let c = Connection::open_with_flags(saida, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|_| "o userData.db está corrompido".to_string())?;
        let ok: String = c.query_row("PRAGMA quick_check", [], |r| r.get(0)).map_err(|_| "o userData.db está corrompido".to_string())?;
        if ok != "ok" {
            return Err(crate::idioma::t("o userData.db está corrompido", "userData.db is corrupted").into());
        }
        for t in TABELAS {
            let existe: i64 = c.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?", [t], |r| r.get(0)).unwrap_or(0);
            if existe == 0 {
                return Err(format!("{} {t}", crate::idioma::t("o userData.db não tem a tabela do JW Library:", "userData.db is missing the JW Library table:")));
            }
        }
        Ok(())
    };
    if let Err(e) = conferir() {
        let _ = fs::remove_file(saida);
        return Err(e);
    }
    let mut r = resumo(saida)?;
    r.aparelho = bloco.get("deviceName").and_then(|v| v.as_str()).unwrap_or("").to_string();
    r.data = bloco.get("lastModifiedDate").and_then(|v| v.as_str()).unwrap_or("").to_string();
    Ok(r)
}

/// Grava o `.jwlibrary` do banco `banco` em `destino`.
pub fn criar(banco: &Path, destino: &Path, aparelho: &str) -> Resultado<Resumo> {
    let (ultima, versao, hoje): (String, i64, String) = {
        let c = Connection::open(banco).map_err(|e| e.to_string())?;
        let _ = c.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        (
            c.query_row("SELECT LastModified FROM LastModified", [], |r| r.get(0)).map_err(|e| e.to_string())?,
            c.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e| e.to_string())?,
            c.query_row("SELECT date('now', 'localtime')", [], |r| r.get(0)).map_err(|e| e.to_string())?,
        )
    };
    let dados = fs::read(banco).map_err(|e| e.to_string())?;
    let nome = destino.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "backup.jwlibrary".into());
    let manifesto = serde_json::json!({
        "name": nome,
        "creationDate": hoje,
        "version": 1,
        "type": 0,
        "userDataBackup": {
            "lastModifiedDate": ultima,
            "deviceName": aparelho,
            "databaseName": BANCO,
            "hash": sha256_hex(&dados),
            "schemaVersion": versao,
        }
    });
    let temporario = destino.with_extension("jwlibrary.parcial");
    {
        let f = fs::File::create(&temporario).map_err(|e| e.to_string())?;
        let mut z = zip::ZipWriter::new(f);
        let opcoes = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (n, conteudo) in [(BANCO, dados.as_slice()), (MINIATURA, IMAGEM), (MANIFESTO, serde_json::to_vec(&manifesto).unwrap().as_slice())] {
            z.start_file(n, opcoes).map_err(|e| e.to_string())?;
            z.write_all(conteudo).map_err(|e| e.to_string())?;
        }
        z.finish().map_err(|e| e.to_string())?;
    }
    fs::rename(&temporario, destino).map_err(|e| e.to_string())?;
    let mut r = resumo(banco)?;
    r.aparelho = aparelho.to_string();
    r.data = ultima;
    Ok(r)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn criar_e_restaurar_dao_o_mesmo() {
        let pasta = std::env::temp_dir().join(format!("leitor-backup-{}", std::process::id()));
        let _ = fs::remove_dir_all(&pasta);
        fs::create_dir_all(&pasta).unwrap();
        let banco = pasta.join("userData.db");
        {
            let p = crate::pessoal::Pessoal::abrir(&banco).unwrap();
            let id = crate::pessoal::Identidade { meps: Some(1), livro: None, capitulo: None, simbolo: "w".into(), edicao: 0, lingua: 5, titulo: "x".into() };
            let g = p.grifar(&id, &[crate::pessoal::Faixa { bloco: 1, inicio: 0, fim: 3 }], 2, None).unwrap();
            p.anotar(&id, None, Some(1), Some(&g), "t", "texto", &["a".into()]).unwrap();
        }
        let arquivo = pasta.join("teste.jwlibrary");
        let r = criar(&banco, &arquivo, "JW Library on Linux").unwrap();
        assert_eq!((r.grifos, r.notas, r.etiquetas), (1, 1, 1));
        let volta = extrair(&arquivo, &pasta.join("volta.db")).unwrap();
        assert_eq!((volta.grifos, volta.notas, volta.etiquetas), (1, 1, 1));
        assert_eq!(volta.aparelho, "JW Library on Linux");
        let _ = fs::remove_dir_all(&pasta);
    }
}
