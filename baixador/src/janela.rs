//! A janela do baixador (Tauri 2): os comandos que a interface em `ui/`
//! chama. O trabalho de verdade continua na lib (o mesmo da linha de
//! comando); aqui só se abre o registro e a seleção, roda o que é demorado
//! em `spawn_blocking` e avisa o andamento por eventos.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::catalogo::{self, CATALOGO, IDIOMAS};
use crate::{Cliente, Desfecho, Escolha, Pastas, Pedido, Registro, Selecao};

struct Estado {
    pastas: Pastas,
    /// Um cliente só, para a regra de uma pergunta por segundo valer para
    /// a janela inteira.
    cliente: Mutex<Cliente>,
    /// Sincronização em andamento (não começa outra, e não se apaga nada).
    ocupado: Arc<AtomicBool>,
}

fn pastas(app: &tauri::AppHandle) -> Pastas {
    app.state::<Estado>().pastas.clone()
}

async fn em_segundo_plano<T: Send + 'static>(
    trabalho: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(trabalho).await.map_err(|e| e.to_string())?
}

#[derive(Serialize)]
struct ItemDoCatalogo {
    sigla: &'static str,
    nome: &'static str,
    categoria: &'static str,
    nome_da_categoria: &'static str,
    periodico: bool,
}

#[derive(Serialize)]
struct Idioma {
    codigo: &'static str,
    nome: &'static str,
}

#[derive(Serialize)]
struct Catalogo {
    itens: Vec<ItemDoCatalogo>,
    idiomas: Vec<Idioma>,
    selecao: Vec<Escolha>,
    /// Chaves do registro (`lff_T`, `mwb_T_202611`), para marcar "baixado".
    baixados: Vec<String>,
    /// Capa de cada sigla_idioma já baixada (a URL que a API mandou).
    capas: std::collections::BTreeMap<String, String>,
}

#[tauri::command]
async fn catalogo(app: tauri::AppHandle) -> Result<Catalogo, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let selecao = Selecao::abrir(&p.selecao())?;
        let mut capas = std::collections::BTreeMap::new();
        for b in registro.baixados.values() {
            if let Some(c) = &b.arquivo.capa {
                capas.insert(format!("{}_{}", b.pedido.sigla, b.pedido.lingua), c.clone());
            }
        }
        Ok(Catalogo {
            itens: CATALOGO
                .iter()
                .map(|i| ItemDoCatalogo {
                    sigla: i.sigla,
                    nome: i.nome,
                    categoria: i.categoria,
                    nome_da_categoria: leitor_nucleo::biblioteca::nome_da_categoria(i.categoria),
                    periodico: i.periodico.is_some(),
                })
                .collect(),
            idiomas: IDIOMAS.iter().map(|(codigo, nome)| Idioma { codigo, nome }).collect(),
            selecao: selecao.escolhas,
            baixados: registro.baixados.keys().cloned().collect(),
            capas,
        })
    })
    .await
}

#[derive(Serialize)]
struct ItemDaSelecao {
    escolha: Escolha,
    chave: String,
    nome: String,
    /// O que já foi baixado desta escolha (nos periódicos, várias edições).
    baixados: Vec<String>,
    capa: Option<String>,
}

#[tauri::command]
async fn selecao(app: tauri::AppHandle) -> Result<Vec<ItemDaSelecao>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let selecao = Selecao::abrir(&p.selecao())?;
        Ok(selecao
            .escolhas
            .into_iter()
            .map(|e| {
                let chave = e.pedido.chave();
                let (sigla, lingua) = (&e.pedido.sigla, &e.pedido.lingua);
                let deste: Vec<&crate::Baixado> = registro
                    .baixados
                    .values()
                    .filter(|b| &b.pedido.sigla == sigla && &b.pedido.lingua == lingua)
                    .filter(|b| e.pedido.edicao.is_empty() || b.pedido.edicao == e.pedido.edicao)
                    .collect();
                let nome = catalogo::achar(sigla)
                    .map(|i| i.nome.to_string())
                    .or_else(|| deste.first().map(|b| b.arquivo.publicacao.clone()))
                    .unwrap_or_else(|| sigla.clone());
                ItemDaSelecao {
                    capa: deste.iter().rev().find_map(|b| b.arquivo.capa.clone()),
                    baixados: deste.iter().map(|b| b.pedido.chave()).collect(),
                    chave,
                    nome,
                    escolha: e,
                }
            })
            .collect())
    })
    .await
}

#[tauri::command]
async fn escolher(app: tauri::AppHandle, sigla: String, lingua: String, edicao: String, novas: bool) -> Result<bool, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let pedido = Pedido::novo(&sigla, &lingua, &edicao);
        if pedido.sigla.is_empty() || pedido.lingua.is_empty() {
            return Err("Falta a sigla ou o idioma.".into());
        }
        let periodico = catalogo::achar(&pedido.sigla).and_then(|i| i.periodico).is_some();
        let novas = novas && periodico && pedido.edicao.is_empty();
        let mut s = Selecao::abrir(&p.selecao())?;
        let mudou = s.escolher(Escolha { pedido, novas_edicoes: novas });
        s.salvar(&p.selecao())?;
        Ok(mudou)
    })
    .await
}

#[tauri::command]
async fn tirar(app: tauri::AppHandle, sigla: String, lingua: String, edicao: String) -> Result<bool, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let mut s = Selecao::abrir(&p.selecao())?;
        let tirou = s.tirar(&Pedido::novo(&sigla, &lingua, &edicao));
        s.salvar(&p.selecao())?;
        Ok(tirou)
    })
    .await
}

#[derive(Serialize)]
struct ItemBaixado {
    chave: String,
    pedido: Pedido,
    publicacao: String,
    titulo: String,
    /// Tamanho do .jwpub no servidor.
    tamanho: u64,
    /// O que a pasta ocupa de verdade na Biblioteca.
    ocupado: u64,
    pasta: String,
    baixado_em: u64,
    capa: Option<String>,
}

fn tamanho_da_pasta(p: &Path) -> u64 {
    let Ok(lista) = std::fs::read_dir(p) else { return 0 };
    lista
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => tamanho_da_pasta(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

#[tauri::command]
async fn baixados(app: tauri::AppHandle) -> Result<Vec<ItemBaixado>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let mut lista: Vec<ItemBaixado> = registro
            .baixados
            .into_iter()
            .map(|(chave, b)| ItemBaixado {
                ocupado: tamanho_da_pasta(&p.biblioteca.join(&b.pasta)),
                chave,
                publicacao: b.arquivo.publicacao,
                titulo: b.arquivo.titulo,
                tamanho: b.arquivo.tamanho,
                capa: b.arquivo.capa,
                pasta: b.pasta,
                baixado_em: b.baixado_em,
                pedido: b.pedido,
            })
            .collect();
        lista.sort_by(|a, b| b.baixado_em.cmp(&a.baixado_em).then(a.chave.cmp(&b.chave)));
        Ok(lista)
    })
    .await
}

#[derive(Clone, Serialize)]
struct Progresso {
    chave: String,
    feito: u64,
    total: u64,
}

/// Começa a sincronização e volta na hora. O andamento chega pelos eventos
/// `progresso` ({chave, feito, total}) e `fim` ({desfechos} ou {erro}).
#[tauri::command]
fn sincronizar(app: tauri::AppHandle) -> Result<(), String> {
    let estado = app.state::<Estado>();
    if estado.ocupado.swap(true, Ordering::SeqCst) {
        return Err("Já está sincronizando.".into());
    }
    let ocupado = estado.ocupado.clone();
    let p = estado.pastas.clone();
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let resultado = (|| -> Result<Vec<Desfecho>, String> {
            let selecao = Selecao::abrir(&p.selecao())?;
            let mut registro = Registro::abrir(&p.registro())?;
            let estado = app2.state::<Estado>();
            let mut cliente = estado.cliente.lock().map_err(|e| e.to_string())?;
            let mut ultimo = (String::new(), u64::MAX);
            Ok(crate::sincronizar(&mut cliente, &p, &mut registro, &selecao, &mut |pedido, feito, total| {
                // Um evento por ponto percentual, não um por bloco lido.
                let chave = pedido.chave();
                let pct = (feito * 100).checked_div(total).unwrap_or(0);
                if ultimo.0 != chave || ultimo.1 != pct {
                    ultimo = (chave.clone(), pct);
                    let _ = app2.emit("progresso", Progresso { chave, feito, total });
                }
            }))
        })();
        ocupado.store(false, Ordering::SeqCst);
        let fim = match resultado {
            Ok(desfechos) => serde_json::json!({ "desfechos": desfechos }),
            Err(erro) => serde_json::json!({ "erro": erro }),
        };
        let _ = app2.emit("fim", fim);
    });
    Ok(())
}

#[tauri::command]
fn sincronizando(app: tauri::AppHandle) -> bool {
    app.state::<Estado>().ocupado.load(Ordering::SeqCst)
}

#[derive(Serialize)]
struct Verificacao {
    novas: Vec<crate::Atualizacao>,
    erros: Vec<String>,
}

#[tauri::command]
async fn verificar(app: tauri::AppHandle) -> Result<Verificacao, String> {
    let p = pastas(&app);
    let app2 = app.clone();
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let estado = app2.state::<Estado>();
        let mut cliente = estado.cliente.lock().map_err(|e| e.to_string())?;
        let (novas, erros) = crate::verificar(&mut cliente, &registro);
        Ok(Verificacao { novas, erros })
    })
    .await
}

/// A pasta da publicação, só se ficar mesmo dentro da Biblioteca.
fn pasta_dentro(biblioteca: &Path, pasta: &str) -> Option<PathBuf> {
    let nome = leitor_nucleo::jwpub::nome_seguro(pasta)?;
    if nome != pasta || nome.is_empty() || nome == "." || nome == ".." {
        return None;
    }
    let base = biblioteca.canonicalize().ok()?;
    let alvo = biblioteca.join(nome).canonicalize().ok()?;
    (alvo.parent() == Some(base.as_path())).then_some(alvo)
}

/// Tira do registro e apaga a pasta na Biblioteca. A seleção fica como está:
/// quem quiser parar de receber tira da seleção.
#[tauri::command]
async fn apagar(app: tauri::AppHandle, chave: String) -> Result<(), String> {
    let estado = app.state::<Estado>();
    if estado.ocupado.load(Ordering::SeqCst) {
        return Err("Espere a sincronização terminar.".into());
    }
    let p = estado.pastas.clone();
    em_segundo_plano(move || {
        let mut registro = Registro::abrir(&p.registro())?;
        let b = registro.baixados.remove(&chave).ok_or("Isso não está no registro.")?;
        if let Some(pasta) = pasta_dentro(&p.biblioteca, &b.pasta) {
            std::fs::remove_dir_all(&pasta).map_err(|e| format!("apagar {}: {e}", pasta.display()))?;
        }
        registro.salvar(&p.registro())
    })
    .await
}

#[derive(Serialize)]
struct Espaco {
    livre: u64,
    total: u64,
}

/// Espaço no disco da Biblioteca (ou da pasta mais perto que já existe).
#[tauri::command]
fn espaco_livre(app: tauri::AppHandle) -> Result<Espaco, String> {
    let mut alvo = pastas(&app).biblioteca;
    while !alvo.exists() {
        if !alvo.pop() {
            return Err("não achei a pasta da Biblioteca".into());
        }
    }
    let c = std::ffi::CString::new(alvo.as_os_str().as_encoded_bytes()).map_err(|e| e.to_string())?;
    // SAFETY: `c` é um caminho terminado em zero e `s` é preenchida pelo statvfs.
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let bloco = s.f_frsize as u64;
    Ok(Espaco { livre: s.f_bavail as u64 * bloco, total: s.f_blocks as u64 * bloco })
}

/// Abre a janela. Chamado pelo `jw-baixador` sem argumentos.
pub fn abrir() -> Result<(), String> {
    let pastas = Pastas::padrao()?;
    tauri::Builder::default()
        .setup(move |app| {
            app.manage(Estado { pastas, cliente: Mutex::new(Cliente::novo()), ocupado: Arc::new(AtomicBool::new(false)) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            catalogo, selecao, escolher, tirar, baixados, sincronizar, sincronizando, verificar, apagar, espaco_livre
        ])
        .run(tauri::generate_context!())
        .map_err(|e| format!("erro ao abrir a janela: {e}"))
}
