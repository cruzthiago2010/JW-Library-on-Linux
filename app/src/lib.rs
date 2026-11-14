//! A casca Tauri do leitor: comandos para a Biblioteca, o sumário e o
//! documento, e o protocolo `conteudo` que serve as imagens das publicações.
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use leitor_nucleo::{biblioteca, jwpub, leitor};
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, UriSchemeContext};

struct Estado {
    raiz: PathBuf,
}

/// O começo do endereço do protocolo `conteudo` na plataforma.
fn base() -> &'static str {
    if cfg!(any(target_os = "android", target_os = "windows")) {
        "http://conteudo.localhost"
    } else {
        "conteudo://localhost"
    }
}

fn raiz(app: &tauri::AppHandle) -> PathBuf {
    app.state::<Estado>().raiz.clone()
}

#[derive(serde::Serialize)]
struct Biblioteca {
    base: &'static str,
    categorias: Vec<biblioteca::Categoria>,
    cartoes: Vec<biblioteca::Cartao>,
}

#[tauri::command]
async fn listar(app: tauri::AppHandle) -> Result<Biblioteca, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let cartoes = biblioteca::cartoes(&raiz);
        Biblioteca { base: base(), categorias: biblioteca::categorias(&cartoes), cartoes }
    })
    .await
    .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct Importacao {
    novas: Vec<String>,
    erros: Vec<String>,
}

#[tauri::command]
async fn importar(app: tauri::AppHandle, caminhos: Vec<String>) -> Result<Importacao, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let mut saida = Importacao { novas: vec![], erros: vec![] };
        for c in caminhos {
            let caminho = Path::new(&c);
            if caminho.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("jwpub")) != Some(true) {
                saida.erros.push(format!("{}: não é um arquivo .jwpub", nome(caminho)));
                continue;
            }
            match jwpub::instalar(caminho, &raiz) {
                Ok(n) => saida.novas.push(n),
                Err(e) => saida.erros.push(format!("{}: {e}", nome(caminho))),
            }
        }
        saida
    })
    .await
    .map_err(|e| e.to_string())
}

fn nome(c: &Path) -> String {
    c.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

#[derive(serde::Serialize)]
struct Sumario {
    pasta: String,
    titulo: String,
    capa: Option<String>,
    ano: i64,
    itens: Vec<jwpub::ItemDoSumario>,
}

#[tauri::command]
async fn sumario(app: tauri::AppHandle, pasta: String) -> Result<Sumario, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let p = jwpub::Publicacao::abrir(&raiz, &pasta)?;
        Ok(Sumario { capa: p.capa(), itens: p.sumario()?, titulo: p.titulo, ano: p.ano, pasta: p.pasta })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct Documento {
    pasta: String,
    publicacao: String,
    id: i64,
    titulo: String,
    html: String,
    anterior: Option<i64>,
    seguinte: Option<i64>,
}

#[tauri::command]
async fn documento(app: tauri::AppHandle, pasta: String, id: i64) -> Result<Documento, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let p = jwpub::Publicacao::abrir(&raiz, &pasta)?;
        let d = p.documento(id)?;
        let ids = p.documentos()?;
        let i = ids.iter().position(|&x| x == id);
        let anterior = i.and_then(|i| i.checked_sub(1)).map(|i| ids[i]);
        let seguinte = i.and_then(|i| ids.get(i + 1).copied());
        Ok(Documento {
            html: leitor::ajustar_html(&d.html, &p.pasta, base()),
            titulo: d.titulo,
            publicacao: p.titulo,
            pasta: p.pasta,
            id,
            anterior,
            seguinte,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Um link para outro documento (pelo número MEPS): em qual publicação instalada ele está.
#[tauri::command]
async fn achar_meps(app: tauri::AppHandle, meps: i64) -> Result<Option<(String, i64)>, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        jwpub::instaladas(&raiz).into_iter().find_map(|p| p.pelo_meps(meps).map(|id| (p.pasta.clone(), id)))
    })
    .await
    .map_err(|e| e.to_string())
}

fn tipo(nome: &str) -> &'static str {
    match nome.rsplit('.').next().map(|e| e.to_ascii_lowercase()).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn servir<R: tauri::Runtime>(ctx: UriSchemeContext<'_, R>, pedido: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let raiz = ctx.app_handle().state::<Estado>().raiz.clone();
    let caminho = pedido.uri().path().trim_start_matches('/').to_string();
    let partes: Vec<&str> = caminho.split('/').collect();
    let arquivo = match partes.as_slice() {
        ["midia", pasta, nome] => match (jwpub::nome_seguro(pasta), jwpub::nome_seguro(nome)) {
            (Some(p), Some(n)) => Some(raiz.join(p).join(jwpub::MIDIA).join(n)),
            _ => None,
        },
        _ => None,
    };
    let resposta = Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    match arquivo.and_then(|a| fs::read(&a).ok().map(|d| (a, d))) {
        Some((a, dados)) => resposta
            .header(header::CONTENT_TYPE, tipo(&a.to_string_lossy()))
            .header(header::CACHE_CONTROL, "max-age=3600")
            .body(Cow::Owned(dados))
            .unwrap(),
        None => resposta.status(StatusCode::NOT_FOUND).body(Cow::Borrowed(&b""[..])).unwrap(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("conteudo", servir)
        .setup(|app| {
            let raiz = app.path().app_data_dir()?.join("biblioteca");
            fs::create_dir_all(&raiz)?;
            app.manage(Estado { raiz });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![listar, importar, sumario, documento, achar_meps])
        .run(tauri::generate_context!())
        .expect("erro ao abrir o app");
}
