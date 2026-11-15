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
pub async fn midia_categoria(app: tauri::AppHandle, lingua: String, chave: String) -> Result<serde_json::Value, String> {
    if !so_codigo(&lingua) || !so_codigo(&chave) {
        return Err("categoria inválida".into());
    }
    fundo(move || {
        if let Some(v) = categoria_local(&app, &lingua, &chave) {
            return Ok(v);
        }
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

// ---------------------------------------------------------------- catálogo de mídia
//
// O mesmo catálogo que o JW Library baixa (`app.jw-cdn.org/catalogs/media/<língua>.json.gz`,
// JSON em linhas): a árvore de categorias e todos os vídeos/áudios (título,
// imagens, duração), sem os endereços dos arquivos. Assim as abas abrem sem
// internet e dá para buscar por nome; o arquivo de cada item vem da API
// `media-items` na hora de tocar ou baixar.

use std::collections::HashMap;
use std::sync::Mutex;

const CATALOGO_MIDIA: &str = "https://app.jw-cdn.org/catalogs/media/";
const MEDIA_ITEMS: &str = "https://b.jw-cdn.org/apis/mediator/v1/media-items";

pub struct CatalogoMidia {
    categorias: Vec<serde_json::Value>,
    itens: HashMap<String, serde_json::Value>,
}

#[derive(Default)]
pub struct CacheMidia(Mutex<HashMap<String, std::sync::Arc<CatalogoMidia>>>);

fn arquivo_catalogo(pasta: &Path, lingua: &str) -> PathBuf {
    pasta.join(format!("catalogo-{lingua}.json"))
}

/// Baixa o catálogo de mídia de uma língua se não há ou se tem mais de 7 dias.
fn garantir_catalogo(pasta: &Path, lingua: &str) -> Result<PathBuf, String> {
    let destino = arquivo_catalogo(pasta, lingua);
    let novo = std::fs::metadata(&destino)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|idade| idade < Duration::from_secs(7 * 86400));
    if novo {
        return Ok(destino);
    }
    let baixar = || -> Result<(), String> {
        std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
        let resp = agente().get(&format!("{CATALOGO_MIDIA}{lingua}.json.gz")).call().map_err(|e| e.to_string())?;
        let mut gz = flate2::read::GzDecoder::new(resp.into_reader());
        let tmp = destino.with_extension("json.tmp");
        let mut saida = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        std::io::copy(&mut gz, &mut saida).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &destino).map_err(|e| e.to_string())
    };
    match baixar() {
        Ok(()) => Ok(destino),
        // Sem internet: fica o que já existe.
        Err(_) if destino.is_file() => Ok(destino),
        Err(e) => Err(e),
    }
}

fn ler_catalogo(caminho: &Path) -> Result<CatalogoMidia, String> {
    let texto = std::fs::read_to_string(caminho).map_err(|e| e.to_string())?;
    let mut c = CatalogoMidia { categorias: vec![], itens: HashMap::new() };
    for linha in texto.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(linha) else { continue };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("category") => c.categorias.push(v["o"].clone()),
            Some("media-item") => {
                if let Some(k) = v["o"].get("naturalKey").and_then(|k| k.as_str()) {
                    c.itens.insert(k.to_string(), v["o"].clone());
                }
            }
            _ => {}
        }
    }
    Ok(c)
}

fn catalogo(app: &tauri::AppHandle, lingua: &str) -> Result<std::sync::Arc<CatalogoMidia>, String> {
    let cache = app.state::<CacheMidia>();
    if let Some(c) = cache.0.lock().map_err(|e| e.to_string())?.get(lingua) {
        return Ok(c.clone());
    }
    let pasta = app.state::<EstadoMidia>().pasta.clone();
    let c = std::sync::Arc::new(ler_catalogo(&garantir_catalogo(&pasta, lingua)?)?);
    cache.0.lock().map_err(|e| e.to_string())?.insert(lingua.to_string(), c.clone());
    Ok(c)
}

fn minutos(segundos: f64) -> String {
    let s = segundos.round() as u64;
    format!("{}m {}s", s / 60, s % 60)
}

/// Um item do catálogo no formato da API (o que a janela já entende), sem `files`.
fn item_para_janela(o: &serde_json::Value) -> serde_json::Value {
    let mut x = o.clone();
    if let Some(m) = x.as_object_mut() {
        let chave = o.get("naturalKey").cloned().unwrap_or_default();
        m.insert("guid".into(), chave);
        if let Some(d) = o.get("duration").and_then(|d| d.as_f64()) {
            m.insert("durationFormattedMinSec".into(), minutos(d).into());
        }
        m.remove("checksums");
    }
    x
}

fn achar<'a>(lista: &'a [serde_json::Value], chave: &str) -> Option<&'a serde_json::Value> {
    for c in lista {
        if c.get("key").and_then(|k| k.as_str()) == Some(chave) {
            return Some(c);
        }
        if let Some(subs) = c.get("subcategories").and_then(|s| s.as_array()) {
            if let Some(a) = achar(subs, chave) {
                return Some(a);
            }
        }
    }
    None
}

/// A categoria com as subcategorias (um nível) e os itens resolvidos.
fn montar(cat: &serde_json::Value, itens: &HashMap<String, serde_json::Value>, nivel: u8) -> serde_json::Value {
    let mut x = cat.clone();
    if let Some(m) = x.as_object_mut() {
        if let Some(chaves) = cat.get("media").and_then(|v| v.as_array()) {
            let lista: Vec<serde_json::Value> =
                chaves.iter().filter_map(|k| k.as_str()).filter_map(|k| itens.get(k)).map(item_para_janela).collect();
            m.insert("media".into(), lista.into());
        }
        if let Some(subs) = cat.get("subcategories").and_then(|v| v.as_array()) {
            let lista: Vec<serde_json::Value> = subs
                .iter()
                .map(|s| if nivel == 0 { montar(s, itens, 1) } else { serde_json::json!({ "key": s["key"], "name": s["name"], "images": s["images"], "type": s["type"] }) })
                .collect();
            m.insert("subcategories".into(), lista.into());
        }
    }
    x
}

/// Uma categoria pelo catálogo local (sem internet, se ele já foi baixado).
pub fn categoria_local(app: &tauri::AppHandle, lingua: &str, chave: &str) -> Option<serde_json::Value> {
    let c = catalogo(app, lingua).ok()?;
    let cat = achar(&c.categorias, chave)?;
    Some(serde_json::json!({ "category": montar(cat, &c.itens, 0) }))
}

fn sem_acento(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

/// Busca vídeos ou áudios pelo título (todas as palavras), os mais novos primeiro.
#[tauri::command]
pub async fn midia_buscar(app: tauri::AppHandle, lingua: String, termos: String, tipo: String) -> Result<Vec<serde_json::Value>, String> {
    if !so_codigo(&lingua) {
        return Err("língua inválida".into());
    }
    fundo(move || {
        let c = catalogo(&app, &lingua)?;
        let palavras: Vec<String> = sem_acento(&termos).split_whitespace().map(str::to_string).collect();
        if palavras.is_empty() {
            return Ok(vec![]);
        }
        let formato = if tipo == "audio" { "AUDIO" } else { "VIDEO" };
        let mut achados: Vec<&serde_json::Value> = c
            .itens
            .values()
            .filter(|o| o.pointer("/keyParts/formatCode").and_then(|f| f.as_str()) == Some(formato))
            .filter(|o| {
                let titulo = sem_acento(o.get("title").and_then(|t| t.as_str()).unwrap_or(""));
                palavras.iter().all(|p| titulo.contains(p.as_str()))
            })
            .collect();
        achados.sort_by(|a, b| b.get("firstPublished").and_then(|v| v.as_str()).cmp(&a.get("firstPublished").and_then(|v| v.as_str())));
        Ok(achados.into_iter().take(120).map(item_para_janela).collect())
    })
    .await
}

/// Os arquivos de um item (vídeo/áudio) pela API, para tocar ou baixar.
#[tauri::command]
pub async fn midia_item(lingua: String, chave: String) -> Result<serde_json::Value, String> {
    if !so_codigo(&lingua) || !so_codigo(&chave) {
        return Err("item inválido".into());
    }
    fundo(move || {
        let texto = agente()
            .get(&format!("{MEDIA_ITEMS}/{lingua}/{chave}"))
            .query("clientType", "www")
            .call()
            .map_err(|e| format!("{}: {e}", t("sem conexão com o jw.org", "no connection to jw.org")))?
            .into_string()
            .map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&texto).map_err(|e| e.to_string())?;
        v.pointer("/media/0").cloned().ok_or_else(|| t("Este item não está disponível.", "This item is not available.").to_string())
    })
    .await
}

// ---------------------------------------------------------------- notícias
//
// A faixa "NOVO VÍDEO | …" do Início do JW Library: `app.jw-cdn.org/apis/alerts`,
// com o token público do próprio app (`tokens/jwl-public.jwt`). O HTML vem do
// jw.org; aqui só passam <p>, <strong>, <em>, <br> e links para jw.org.

const TOKEN: &str = "https://app.jw-cdn.org/tokens/jwl-public.jwt";
const ALERTAS: &str = "https://app.jw-cdn.org/apis/alerts/list";

#[derive(Serialize)]
pub struct Noticia {
    guid: String,
    titulo: String,
    corpo: String,
}

/// Só as marcas permitidas; links só para jw.org (com data-fora para abrir fora).
fn limpar_html(html: &str) -> String {
    let mut saida = String::new();
    let mut resto = html;
    while let Some(i) = resto.find('<') {
        saida.push_str(&resto[..i]);
        let Some(j) = resto[i..].find('>') else { break };
        let marca = &resto[i + 1..i + j];
        let fecha = marca.starts_with('/');
        let nome: String = marca.trim_start_matches('/').chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>().to_lowercase();
        match nome.as_str() {
            "p" | "strong" | "em" | "b" | "i" | "br" => saida.push_str(&format!("<{}{}>", if fecha { "/" } else { "" }, nome)),
            "a" if fecha => saida.push_str("</a>"),
            "a" => {
                let href = marca.split("href=\"").nth(1).and_then(|r| r.split('"').next()).unwrap_or("").replace("&amp;", "&");
                if href.starts_with("https://www.jw.org/") || href.starts_with("https://wol.jw.org/") {
                    let seguro: String = href.chars().filter(|c| !"\"<>'`".contains(*c)).collect();
                    saida.push_str(&format!("<a href=\"#\" data-fora=\"{seguro}\">"));
                } else {
                    saida.push_str("<a>");
                }
            }
            _ => {}
        }
        resto = &resto[i + j + 1..];
    }
    saida.push_str(resto);
    saida
}

#[tauri::command]
pub async fn noticias(lingua: String) -> Result<Vec<Noticia>, String> {
    if !so_codigo(&lingua) {
        return Err("língua inválida".into());
    }
    fundo(move || {
        let ag = agente();
        let token = ag.get(TOKEN).call().map_err(|e| e.to_string())?.into_string().map_err(|e| e.to_string())?;
        let texto = ag
            .get(ALERTAS)
            .query("type", "news")
            .query("lang", &lingua)
            .set("Authorization", &format!("Bearer {}", token.trim()))
            .call()
            .map_err(|e| e.to_string())?
            .into_string()
            .map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&texto).map_err(|e| e.to_string())?;
        Ok(v.get("alerts")
            .and_then(|a| a.as_array())
            .map(|lista| {
                lista
                    .iter()
                    .map(|a| Noticia {
                        guid: a.get("guid").and_then(|g| g.as_str()).unwrap_or("").to_string(),
                        titulo: limpar_html(a.get("title").and_then(|g| g.as_str()).unwrap_or("")),
                        corpo: limpar_html(a.get("body").and_then(|g| g.as_str()).unwrap_or("")),
                    })
                    .collect()
            })
            .unwrap_or_default())
    })
    .await
}

/// Abre um link do jw.org no navegador do sistema.
#[tauri::command]
pub fn abrir_link(url: String) -> Result<(), String> {
    if !(url.starts_with("https://www.jw.org/") || url.starts_with("https://wol.jw.org/")) || url.chars().any(|c| c.is_whitespace()) {
        return Err("link fora do jw.org".into());
    }
    std::process::Command::new("xdg-open").arg(url).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::limpar_html;

    #[test]
    fn limpa_o_html_das_noticias() {
        let h = limpar_html(r#"<strong>NOVO VÍDEO</strong> | <em>x</em><script>alert(1)</script><a href="https://www.jw.org/finder?lank=a&amp;b=c" onclick="y">ver</a><img src=x onerror=z><a href="javascript:1">j</a>"#);
        assert_eq!(h, r##"<strong>NOVO VÍDEO</strong> | <em>x</em>alert(1)<a href="#" data-fora="https://www.jw.org/finder?lank=a&b=c">ver</a><a>j</a>"##);
    }
}
