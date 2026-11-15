//! As abas VÍDEO e ÁUDIO da Biblioteca: as categorias do jw.org (API
//! "mediator", a mesma do site), a Bíblia em áudio (API de publicações em MP3)
//! e os arquivos baixados para ver/ouvir sem internet, servidos pelo protocolo
//! `conteudo://localhost/video/<nome>` (com Range, para dar para avançar).
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use leitor_nucleo::idioma::t;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

const MEDIATOR: &str = "https://b.jw-cdn.org/apis/mediator/v1/categories";
const PUBMEDIA: &str = "https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS";

pub struct EstadoMidia {
    pub pasta: PathBuf,
    baixando: AtomicBool,
}

impl EstadoMidia {
    pub fn novo(dados: &Path) -> Self {
        EstadoMidia { pasta: dados.join("midia"), baixando: AtomicBool::new(false) }
    }
}

fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent(leitor_baixar::api::AGENTE)
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(120))
        .build()
}

fn so_codigo(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

async fn fundo<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

/// Uma categoria do jw.org (com as subcategorias e os vídeos/áudios). A janela
/// recebe o JSON como vem.
#[tauri::command]
pub async fn midia_categoria(lingua: String, chave: String) -> Result<serde_json::Value, String> {
    if !so_codigo(&lingua) || !so_codigo(&chave) {
        return Err("categoria inválida".into());
    }
    fundo(move || {
        let texto = agente()
            .get(&format!("{MEDIATOR}/{lingua}/{chave}"))
            .query("detailed", "1")
            .query("clientType", "www")
            .call()
            .map_err(|e| format!("{}: {e}", t("sem conexão com o jw.org", "no connection to jw.org")))?
            .into_string()
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&texto).map_err(|e| e.to_string())
    })
    .await
}

#[derive(Serialize)]
pub struct Capitulo {
    titulo: String,
    capitulo: i64,
    url: String,
    duracao: f64,
    tamanho: u64,
}

/// Os capítulos de um livro da Bíblia em áudio (MP3).
#[tauri::command]
pub async fn midia_biblia(lingua: String, livro: u32) -> Result<Vec<Capitulo>, String> {
    if !so_codigo(&lingua) || !(1..=66).contains(&livro) {
        return Err("livro inválido".into());
    }
    fundo(move || {
        let texto = agente()
            .get(PUBMEDIA)
            .query("output", "json")
            .query("pub", "nwt")
            .query("langwritten", &lingua)
            .query("fileformat", "MP3")
            .query("booknum", &livro.to_string())
            .call()
            .map_err(|e| format!("{}: {e}", t("sem conexão com o jw.org", "no connection to jw.org")))?
            .into_string()
            .map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&texto).map_err(|e| e.to_string())?;
        let lista = v.pointer(&format!("/files/{lingua}/MP3")).and_then(|x| x.as_array()).cloned().unwrap_or_default();
        Ok(lista
            .iter()
            .filter_map(|x| {
                Some(Capitulo {
                    titulo: x.get("title")?.as_str()?.to_string(),
                    capitulo: x.get("track")?.as_i64()?,
                    url: x.pointer("/file/url")?.as_str()?.to_string(),
                    duracao: x.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0),
                    tamanho: x.get("filesize").and_then(|d| d.as_u64()).unwrap_or(0),
                })
            })
            .filter(|c| c.capitulo > 0)
            .collect())
    })
    .await
}

// ---------------------------------------------------------------- baixados

#[derive(Serialize, Deserialize, Clone)]
pub struct Baixada {
    pub nome: String,
    pub titulo: String,
    pub imagem: Option<String>,
    pub tipo: String, // "video" | "audio"
    pub tamanho: u64,
    pub baixada_em: u64,
}

fn indice(pasta: &Path) -> PathBuf {
    pasta.join("indice.json")
}

fn ler_indice(pasta: &Path) -> Vec<Baixada> {
    std::fs::read_to_string(indice(pasta)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn gravar_indice(pasta: &Path, lista: &[Baixada]) -> Result<(), String> {
    std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
    let tmp = pasta.join("indice.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(lista).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, indice(pasta)).map_err(|e| e.to_string())
}

/// O nome do arquivo no fim da URL, só se a URL for do jw-cdn.
fn nome_da_url(url: &str) -> Option<String> {
    let resto = url.strip_prefix("https://")?;
    let (host, caminho) = resto.split_once('/')?;
    if !(host == "jw-cdn.org" || host.ends_with(".jw-cdn.org")) {
        return None;
    }
    let nome = caminho.split(['?', '#']).next()?.rsplit('/').next()?.to_string();
    let ok = nome.len() <= 120
        && nome.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        && !nome.starts_with('.')
        && [".mp4", ".mp3", ".m4v", ".3gp"].iter().any(|e| nome.to_ascii_lowercase().ends_with(e));
    ok.then_some(nome)
}

#[tauri::command]
pub fn midia_baixadas(app: tauri::AppHandle) -> Vec<Baixada> {
    let pasta = app.state::<EstadoMidia>().pasta.clone();
    ler_indice(&pasta).into_iter().filter(|b| pasta.join(&b.nome).is_file()).collect()
}

#[derive(Clone, Serialize)]
struct Progresso {
    nome: String,
    feito: u64,
    total: u64,
}

/// Baixa um vídeo ou áudio do jw-cdn para ver sem internet. Andamento por
/// `midia-progresso`, fim por `midia-fim` ({nome} ou {erro}).
#[tauri::command]
pub fn midia_baixar(app: tauri::AppHandle, url: String, titulo: String, imagem: Option<String>, tipo: String) -> Result<String, String> {
    let nome = nome_da_url(&url).ok_or("endereço de vídeo/áudio inválido")?;
    let estado = app.state::<EstadoMidia>();
    if estado.baixando.swap(true, Ordering::SeqCst) {
        return Err(t("Espere o download atual terminar.", "Wait for the current download to finish.").into());
    }
    let pasta = estado.pasta.clone();
    let nome2 = nome.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let resultado = (|| -> Result<u64, String> {
            std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
            let resp = agente().get(&url).call().map_err(|e| format!("{}: {e}", t("sem conexão", "no connection")))?;
            let total: u64 = resp.header("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
            let parcial = pasta.join(format!("{nome2}.part"));
            let mut saida = std::fs::File::create(&parcial).map_err(|e| e.to_string())?;
            let mut leitor = resp.into_reader();
            let mut buf = vec![0u8; 128 * 1024];
            let (mut feito, mut ultimo) = (0u64, u64::MAX);
            loop {
                let n = leitor.read(&mut buf).map_err(|e| format!("download interrompido: {e}"))?;
                if n == 0 {
                    break;
                }
                saida.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                feito += n as u64;
                let pct = (feito * 100).checked_div(total).unwrap_or(0);
                if pct != ultimo {
                    ultimo = pct;
                    let _ = app.emit("midia-progresso", Progresso { nome: nome2.clone(), feito, total });
                }
            }
            if total > 0 && feito != total {
                let _ = std::fs::remove_file(&parcial);
                return Err(format!("download incompleto: {feito} de {total} bytes"));
            }
            std::fs::rename(&parcial, pasta.join(&nome2)).map_err(|e| e.to_string())?;
            Ok(feito)
        })();
        match resultado {
            Ok(tamanho) => {
                let mut lista = ler_indice(&pasta);
                lista.retain(|b| b.nome != nome2);
                lista.insert(0, Baixada { nome: nome2.clone(), titulo, imagem, tipo, tamanho, baixada_em: leitor_baixar::agora() });
                let _ = gravar_indice(&pasta, &lista);
                let _ = app.emit("midia-fim", serde_json::json!({ "nome": nome2 }));
            }
            Err(erro) => {
                let _ = app.emit("midia-fim", serde_json::json!({ "nome": nome2, "erro": erro }));
            }
        }
        app.state::<EstadoMidia>().baixando.store(false, Ordering::SeqCst);
    });
    Ok(nome)
}

#[tauri::command]
pub fn midia_apagar(app: tauri::AppHandle, nome: String) -> Result<(), String> {
    let pasta = app.state::<EstadoMidia>().pasta.clone();
    let mut lista = ler_indice(&pasta);
    if !lista.iter().any(|b| b.nome == nome) {
        return Err("arquivo desconhecido".into());
    }
    let _ = std::fs::remove_file(pasta.join(&nome));
    lista.retain(|b| b.nome != nome);
    gravar_indice(&pasta, &lista)
}

/// Abre no reprodutor do sistema (se o do app não tocar o formato).
#[tauri::command]
pub fn midia_abrir_fora(app: tauri::AppHandle, nome: Option<String>, url: Option<String>) -> Result<(), String> {
    let alvo = match (nome, url) {
        (Some(n), _) => {
            let pasta = app.state::<EstadoMidia>().pasta.clone();
            if !ler_indice(&pasta).iter().any(|b| b.nome == n) {
                return Err("arquivo desconhecido".into());
            }
            pasta.join(n).to_string_lossy().into_owned()
        }
        (None, Some(u)) if nome_da_url(&u).is_some() => u,
        _ => return Err("nada para abrir".into()),
    };
    std::process::Command::new("xdg-open").arg(alvo).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// `conteudo://localhost/video/<nome>`: o arquivo baixado, com suporte a Range.
pub fn servir_arquivo(pasta: &Path, nome: &str, range: Option<&str>) -> Option<(u16, Vec<(String, String)>, Vec<u8>)> {
    if !ler_indice(pasta).iter().any(|b| b.nome == nome) {
        return None;
    }
    let caminho = pasta.join(nome);
    let mut f = std::fs::File::open(&caminho).ok()?;
    let total = f.metadata().ok()?.len();
    let tipo = if nome.ends_with(".mp3") { "audio/mpeg" } else { "video/mp4" };
    let mut cab = vec![("Content-Type".to_string(), tipo.to_string()), ("Accept-Ranges".to_string(), "bytes".to_string())];
    let faixa = range.and_then(|r| r.strip_prefix("bytes=")).and_then(|r| {
        let (a, b) = r.split_once('-')?;
        let ini: u64 = a.parse().ok()?;
        let fim: u64 = if b.is_empty() { (ini + 4 * 1024 * 1024 - 1).min(total - 1) } else { b.parse::<u64>().ok()?.min(total - 1) };
        (ini <= fim).then_some((ini, fim))
    });
    match faixa {
        Some((ini, fim)) => {
            let mut dados = vec![0u8; (fim - ini + 1) as usize];
            f.seek(SeekFrom::Start(ini)).ok()?;
            f.read_exact(&mut dados).ok()?;
            cab.push(("Content-Range".into(), format!("bytes {ini}-{fim}/{total}")));
            Some((206, cab, dados))
        }
        None => {
            let mut dados = vec![];
            f.read_to_end(&mut dados).ok()?;
            Some((200, cab, dados))
        }
    }
}

// ---------------------------------------------------------------- servidor local
//
// O WebKitGTK não toca vídeo de protocolo próprio (`conteudo://`): o reprodutor
// (GStreamer) só entende http/arquivo. Então os baixados saem por um servidor
// mínimo em 127.0.0.1, porta aleatória, com uma chave aleatória no caminho.

pub struct Servidor {
    pub base: String,
}

pub fn iniciar_servidor(pasta: PathBuf) -> Result<Servidor, String> {
    let ouvinte = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let porta = ouvinte.local_addr().map_err(|e| e.to_string())?.port();
    let chave: String = format!("{:x}{:x}", std::process::id(), leitor_baixar::agora() ^ (porta as u64) << 17)
        .chars()
        .chain(uuid_simples())
        .take(24)
        .collect();
    let base = format!("http://127.0.0.1:{porta}/{chave}");
    std::thread::spawn(move || {
        for conexao in ouvinte.incoming().flatten() {
            let pasta = pasta.clone();
            let chave = chave.clone();
            std::thread::spawn(move || {
                let _ = atender(conexao, &pasta, &chave);
            });
        }
    });
    Ok(Servidor { base })
}

fn uuid_simples() -> impl Iterator<Item = char> {
    let mut x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(7) as u64;
    (0..16).map(move |_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        char::from_digit((x % 16) as u32, 16).unwrap()
    })
}

fn atender(mut c: std::net::TcpStream, pasta: &Path, chave: &str) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader};
    c.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut leitor = BufReader::new(c.try_clone()?);
    let mut primeira = String::new();
    leitor.read_line(&mut primeira)?;
    let mut range = None;
    loop {
        let mut linha = String::new();
        if leitor.read_line(&mut linha)? == 0 || linha == "\r\n" || linha == "\n" {
            break;
        }
        if let Some((k, v)) = linha.split_once(':') {
            if k.eq_ignore_ascii_case("range") {
                range = Some(v.trim().to_string());
            }
        }
    }
    let mut partes = primeira.split_whitespace();
    let metodo = partes.next().unwrap_or("");
    let caminho = partes.next().unwrap_or("");
    let nome = caminho.strip_prefix('/').and_then(|r| r.strip_prefix(chave)).and_then(|r| r.strip_prefix("/video/"));
    let resposta = match (metodo, nome.and_then(leitor_nucleo::jwpub::nome_seguro)) {
        ("GET" | "HEAD", Some(n)) => servir_arquivo(pasta, &n, range.as_deref()),
        _ => None,
    };
    match resposta {
        Some((status, cab, dados)) => {
            let texto = if status == 206 { "Partial Content" } else { "OK" };
            let mut s = format!("HTTP/1.1 {status} {texto}\r\nContent-Length: {}\r\nConnection: close\r\n", dados.len());
            for (k, v) in cab {
                s.push_str(&format!("{k}: {v}\r\n"));
            }
            s.push_str("\r\n");
            c.write_all(s.as_bytes())?;
            if metodo == "GET" {
                c.write_all(&dados)?;
            }
        }
        None => c.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")?,
    }
    c.flush()
}

#[tauri::command]
pub fn midia_base(app: tauri::AppHandle) -> Option<String> {
    app.try_state::<Servidor>().map(|s| s.base.clone())
}
