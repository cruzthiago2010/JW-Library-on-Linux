//! A casca Tauri do leitor: comandos para a Biblioteca, o sumário e o
//! documento, e o protocolo `conteudo` que serve as imagens das publicações.
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use std::sync::Mutex;

use leitor_nucleo::pessoal::{self, Alvo, Faixa, Identidade, Pessoal};
use leitor_nucleo::{backup, biblia, biblioteca, busca, estudo, jwpub, leitor, reunioes};
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, UriSchemeContext};

struct Estado {
    raiz: PathBuf,
    dados: PathBuf,
    pessoal: Mutex<Pessoal>,
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
                saida.erros.push(format!("{}: {}", nome(caminho), leitor_nucleo::idioma::t("não é um arquivo .jwpub", "not a .jwpub file")));
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

#[tauri::command]
async fn remover_publicacao(app: tauri::AppHandle, pasta: String) -> Result<(), String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || jwpub::remover(&raiz, &pasta)).await.map_err(|e| e.to_string())?
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

#[tauri::command]
async fn painel(app: tauri::AppHandle, pasta: String, id: i64) -> Result<estudo::Painel, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let p = jwpub::Publicacao::abrir(&raiz, &pasta)?;
        estudo::painel(&p, id, base())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// A semana que contém `data` (AAAAMMDD): apostila e Sentinela instaladas.
#[tauri::command]
async fn semana(app: tauri::AppHandle, data: i64) -> Result<reunioes::Semana, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || reunioes::semana(&raiz, data))
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct Novo {
    caminho: String,
    nome: String,
}

/// Arquivos .jwpub na pasta Downloads que ainda não estão na biblioteca
/// (ou que mudaram de tamanho desde a importação).
#[tauri::command]
async fn novos_em_downloads(app: tauri::AppHandle) -> Result<Vec<Novo>, String> {
    let raiz = raiz(&app);
    let Ok(downloads) = app.path().download_dir() else { return Ok(vec![]) };
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(itens) = fs::read_dir(&downloads) else { return vec![] };
        let mut novos: Vec<Novo> = itens
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|c| c.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("jwpub")) == Some(true))
            .filter(|c| {
                let Some(pasta) = c.file_stem().and_then(|s| s.to_str()).and_then(jwpub::nome_seguro) else { return false };
                let marca = raiz.join(pasta).join("instalado.json");
                let tamanho = fs::metadata(c).map(|m| m.len()).unwrap_or(0);
                let instalado = fs::read_to_string(marca)
                    .ok()
                    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                    .and_then(|j| j.get("tamanho").and_then(|t| t.as_u64()));
                instalado != Some(tamanho)
            })
            .map(|c| Novo { nome: nome(&c), caminho: c.to_string_lossy().into_owned() })
            .collect();
        novos.sort_by(|a, b| a.nome.cmp(&b.nome));
        novos
    })
    .await
    .map_err(|e| e.to_string())
}

fn achar_biblia(raiz: &Path) -> Result<biblia::Biblia, String> {
    biblia::Biblia::achar(raiz).ok_or_else(|| leitor_nucleo::idioma::t("Nenhuma Bíblia instalada", "No Bible installed").to_string())
}

#[derive(serde::Serialize)]
struct Livros {
    publicacao: String,
    livros: Vec<biblia::Livro>,
}

#[tauri::command]
async fn livros(app: tauri::AppHandle) -> Result<Livros, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let b = achar_biblia(&raiz)?;
        Ok(Livros { livros: b.livros()?, publicacao: b.publicacao.titulo })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct Capitulo {
    livro: u32,
    capitulo: u32,
    titulo: String,
    publicacao: String,
    html: String,
    notas: Vec<biblia::NotaDeEstudo>,
    anterior: Option<(u32, u32)>,
    seguinte: Option<(u32, u32)>,
}

#[tauri::command]
async fn capitulo(app: tauri::AppHandle, livro: u32, capitulo: u32) -> Result<Capitulo, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let b = achar_biblia(&raiz)?;
        let livros = b.livros()?;
        let total = |l: u32| livros.get(l as usize - 1).map(|x| x.capitulos).unwrap_or(1);
        let anterior = if capitulo > 1 { Some((livro, capitulo - 1)) } else if livro > 1 { Some((livro - 1, total(livro - 1))) } else { None };
        let seguinte = if capitulo < total(livro) { Some((livro, capitulo + 1)) } else if livro < 66 { Some((livro + 1, 1)) } else { None };
        let html = b.capitulo(livro, capitulo)?;
        Ok(Capitulo {
            html: leitor::ajustar_html(&html, &b.publicacao.pasta, base()),
            notas: b.notas_de_estudo(livro, capitulo, base())?,
            titulo: format!("{} {capitulo}", biblia::nome(livro)),
            publicacao: b.publicacao.titulo,
            livro,
            capitulo,
            anterior,
            seguinte,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn referencias(app: tauri::AppHandle, livro: u32, capitulo: u32, mid: i64) -> Result<Vec<biblia::Referencia>, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || achar_biblia(&raiz)?.referencias_com_texto(livro, capitulo, mid))
        .await
        .map_err(|e| e.to_string())?
}

/// O texto de um link bíblico de artigo (`NWTR/43:3:16-43:3:18`) ou de uma faixa `inicio-fim`.
#[tauri::command]
async fn texto_biblico(app: tauri::AppHandle, link: String) -> Result<biblia::Referencia, String> {
    let raiz = raiz(&app);
    tauri::async_runtime::spawn_blocking(move || {
        let (a, b) = biblia::faixa_do_link(&link).or_else(|| biblia::faixa(&link)).ok_or(leitor_nucleo::idioma::t("referência inválida", "invalid reference"))?;
        achar_biblia(&raiz)?.referencia(a, b)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct Busca {
    total: usize,
    achados: Vec<busca::Achado>,
}

#[tauri::command]
async fn buscar_biblia(app: tauri::AppHandle, termos: String, exata: bool) -> Result<Busca, String> {
    let raiz = raiz(&app);
    let indice = app.state::<Estado>().dados.join("busca.db");
    tauri::async_runtime::spawn_blocking(move || {
        let (total, achados) = busca::buscar(&raiz, &indice, &termos, exata, 300)?;
        Ok(Busca { total, achados })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------- estudo pessoal

fn identidade(raiz: &Path, alvo: &Alvo) -> Result<Identidade, String> {
    match alvo {
        Alvo::Doc { pasta, id } => Identidade::de_documento(&jwpub::Publicacao::abrir(raiz, pasta)?, *id),
        Alvo::Biblia { livro, capitulo } => {
            let b = achar_biblia(raiz)?;
            let nome = biblia::nome(*livro);
            Ok(Identidade::de_capitulo(&b.publicacao, *livro, *capitulo, format!("{nome} {capitulo}")))
        }
    }
}

/// Roda `f` com o banco do usuário e a identidade do lugar.
fn com_pessoal<T>(app: &tauri::AppHandle, alvo: &Alvo, f: impl FnOnce(&Pessoal, &Identidade) -> Result<T, String>) -> Result<T, String> {
    let estado = app.state::<Estado>();
    let id = identidade(&estado.raiz, alvo)?;
    let p = estado.pessoal.lock().map_err(|e| e.to_string())?;
    f(&p, &id)
}

#[tauri::command]
fn marcas_do_lugar(app: tauri::AppHandle, alvo: Alvo) -> Result<pessoal::DoLugar, String> {
    com_pessoal(&app, &alvo, |p, id| p.do_lugar(id))
}

#[tauri::command]
fn grifar(app: tauri::AppHandle, alvo: Alvo, faixas: Vec<Faixa>, cor: i64, guid: Option<String>) -> Result<String, String> {
    com_pessoal(&app, &alvo, |p, id| p.grifar(id, &faixas, cor, guid))
}

#[tauri::command]
fn remover_grifo(app: tauri::AppHandle, guid: String) -> Result<(), String> {
    app.state::<Estado>().pessoal.lock().map_err(|e| e.to_string())?.remover_grifo(&guid)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn anotar(app: tauri::AppHandle, alvo: Alvo, guid: Option<String>, bloco: Option<i64>, grifo: Option<String>, titulo: String, texto: String, etiquetas: Vec<String>) -> Result<String, String> {
    com_pessoal(&app, &alvo, |p, id| p.anotar(id, guid, bloco, grifo.as_deref(), &titulo, &texto, &etiquetas))
}

#[tauri::command]
fn apagar_nota(app: tauri::AppHandle, guid: String) -> Result<(), String> {
    app.state::<Estado>().pessoal.lock().map_err(|e| e.to_string())?.apagar_nota(&guid)
}

#[tauri::command]
fn campo(app: tauri::AppHandle, alvo: Alvo, tag: String, valor: String) -> Result<(), String> {
    com_pessoal(&app, &alvo, |p, id| p.campo(id, &tag, &valor))
}

#[tauri::command]
fn notas(app: tauri::AppHandle, filtro: String, etiqueta: Option<String>) -> Result<Vec<pessoal::NotaComOrigem>, String> {
    let estado = app.state::<Estado>();
    let mut lista = estado.pessoal.lock().map_err(|e| e.to_string())?.todas_as_notas(&filtro, etiqueta.as_deref())?;
    // A origem legível: "Romanos 14", ou "Título do artigo — Publicação" se estiver instalada.
    let instaladas = jwpub::instaladas(&estado.raiz);
    for n in lista.iter_mut() {
        if let (Some(l), Some(c)) = (n.livro, n.capitulo) {
            n.origem = format!("{} {c}", biblia::nome(l as u32));
        } else if let Some(meps) = n.meps {
            if let Some((p, id)) = instaladas.iter().find_map(|p| p.pelo_meps(meps).map(|id| (p, id))) {
                let titulo = p.documento(id).map(|d| d.titulo).unwrap_or_default();
                n.origem = format!("{titulo} — {}", p.titulo);
            } else if n.origem.is_empty() {
                n.origem = format!("{} ({})", n.simbolo, leitor_nucleo::idioma::t("não instalada", "not installed"));
            }
        }
    }
    Ok(lista)
}

#[tauri::command]
fn etiquetas(app: tauri::AppHandle) -> Result<Vec<(String, i64)>, String> {
    app.state::<Estado>().pessoal.lock().map_err(|e| e.to_string())?.etiquetas()
}

#[tauri::command]
fn apagar_etiqueta(app: tauri::AppHandle, nome: String) -> Result<(), String> {
    app.state::<Estado>().pessoal.lock().map_err(|e| e.to_string())?.apagar_etiqueta(&nome)
}

#[tauri::command]
fn marcadores(app: tauri::AppHandle, alvo: Alvo) -> Result<Vec<pessoal::Marcador>, String> {
    com_pessoal(&app, &alvo, |p, id| p.marcadores(id))
}

#[tauri::command]
fn marcar(app: tauri::AppHandle, alvo: Alvo, slot: i64, bloco: Option<i64>, trecho: String) -> Result<(), String> {
    com_pessoal(&app, &alvo, |p, id| p.marcar(id, slot, bloco, &trecho))
}

#[tauri::command]
fn desmarcar(app: tauri::AppHandle, alvo: Alvo, slot: i64) -> Result<(), String> {
    com_pessoal(&app, &alvo, |p, id| p.desmarcar(id, slot))
}

// ---------------------------------------------------------------- backup

/// O idioma da interface ("pt" ou "en"), escolhido na janela.
#[tauri::command]
fn definir_idioma(idioma: String) {
    leitor_nucleo::idioma::definir(&idioma);
}

#[tauri::command]
fn resumo_pessoal(app: tauri::AppHandle) -> Result<backup::Resumo, String> {
    let estado = app.state::<Estado>();
    let _trava = estado.pessoal.lock().map_err(|e| e.to_string())?;
    backup::resumo(&estado.dados.join("userData.db"))
}

#[tauri::command]
fn criar_backup(app: tauri::AppHandle, destino: String) -> Result<backup::Resumo, String> {
    let estado = app.state::<Estado>();
    let _trava = estado.pessoal.lock().map_err(|e| e.to_string())?;
    let mut destino = PathBuf::from(destino);
    if destino.extension().and_then(|e| e.to_str()) != Some("jwlibrary") {
        destino.set_extension("jwlibrary");
    }
    backup::criar(&estado.dados.join("userData.db"), &destino, "JW Library on Linux")
}

/// Confere o backup e mostra o que tem, sem trocar nada ainda.
#[tauri::command]
fn conferir_backup(app: tauri::AppHandle, arquivo: String) -> Result<backup::Resumo, String> {
    let estado = app.state::<Estado>();
    backup::extrair(Path::new(&arquivo), &estado.dados.join("userData.restaurar.db"))
}

/// Troca o estudo pessoal pelo do backup conferido. O de antes fica em `userData.antes.db`.
#[tauri::command]
fn restaurar_backup(app: tauri::AppHandle) -> Result<backup::Resumo, String> {
    let estado = app.state::<Estado>();
    let novo = estado.dados.join("userData.restaurar.db");
    if !novo.exists() {
        return Err(leitor_nucleo::idioma::t("confira o backup antes de restaurar", "check the backup before restoring").into());
    }
    let atual = estado.dados.join("userData.db");
    let mut p = estado.pessoal.lock().map_err(|e| e.to_string())?;
    // Fecha o banco atual antes de trocar o arquivo.
    *p = Pessoal { conexao: rusqlite_em_memoria()? };
    let _ = fs::copy(&atual, estado.dados.join("userData.antes.db"));
    fs::rename(&novo, &atual).map_err(|e| e.to_string())?;
    *p = Pessoal::abrir(&atual)?;
    backup::resumo(&atual)
}

fn rusqlite_em_memoria() -> Result<leitor_nucleo::pessoal::Conexao, String> {
    leitor_nucleo::pessoal::Conexao::open_in_memory().map_err(|e| e.to_string())
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
            let dados = app.path().app_data_dir()?;
            let raiz = dados.join("biblioteca");
            fs::create_dir_all(&raiz)?;
            let pessoal = Pessoal::abrir(&dados.join("userData.db")).map_err(|e| e.to_string())?;
            app.manage(Estado { raiz, dados, pessoal: Mutex::new(pessoal) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![listar, importar, sumario, documento, achar_meps, painel, semana, novos_em_downloads,
            livros, capitulo, referencias, texto_biblico, buscar_biblia,
            marcas_do_lugar, grifar, remover_grifo, anotar, apagar_nota, campo, notas, etiquetas, apagar_etiqueta,
            marcadores, marcar, desmarcar, definir_idioma, remover_publicacao, resumo_pessoal, criar_backup, conferir_backup, restaurar_backup
        ])
        .run(tauri::generate_context!())
        .expect("erro ao abrir o app");
}
