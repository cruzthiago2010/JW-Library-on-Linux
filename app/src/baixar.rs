//! A aba "Baixar" da Biblioteca: catálogo, seleção, sincronização e procura
//! de atualizações (o antigo Baixador, agora embutido). O trabalho fica no
//! crate `leitor-baixar`; aqui só os comandos, o andamento por eventos
//! (`baixar-progresso`, `baixar-fim`) e a trava de uma tarefa por vez.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use leitor_baixar::catalogo::{self, CATALOGO, IDIOMAS};
use leitor_baixar::{Atualizacao, Cliente, Desfecho, Escolha, Pastas, Pedido, Registro, Selecao};
use leitor_nucleo::idioma::t;
use serde::Serialize;
use tauri::{Emitter, Manager};

pub struct EstadoBaixar {
    pub pastas: Pastas,
    cliente: Mutex<Cliente>,
    ocupado: Arc<AtomicBool>,
    procurando: Arc<AtomicBool>,
    pendentes: Mutex<Vec<Atualizacao>>,
}

impl EstadoBaixar {
    /// `dados` é a pasta do leitor (`~/.local/share/io.github.leitorjw`): a
    /// Biblioteca fica em `biblioteca/` e o registro e a seleção em `baixar/`.
    /// Na primeira vez, traz a seleção e o registro do Baixador separado.
    pub fn novo(dados: &Path) -> Self {
        let pastas = Pastas { biblioteca: dados.join("biblioteca"), dados: dados.join("baixar") };
        trazer_do_baixador_antigo(&pastas);
        EstadoBaixar {
            pastas,
            cliente: Mutex::new(Cliente::novo()),
            ocupado: Arc::new(AtomicBool::new(false)),
            procurando: Arc::new(AtomicBool::new(false)),
            pendentes: Mutex::new(vec![]),
        }
    }
}

/// As pastas onde o Baixador separado guardava a seleção e o registro
/// (fora do Flatpak e no Flatpak dele).
fn pastas_do_baixador_antigo() -> Vec<PathBuf> {
    let Some(casa) = std::env::var_os("HOME").map(PathBuf::from) else { return vec![] };
    vec![
        casa.join(".local/share/io.github.leitorjw.baixador"),
        casa.join(".var/app/io.github.cruzthiago2010.JWLibraryDownloaderOnLinux/data/io.github.leitorjw.baixador"),
    ]
}

fn trazer_do_baixador_antigo(p: &Pastas) {
    if p.selecao().exists() || p.registro().exists() {
        return;
    }
    let Some(antiga) = pastas_do_baixador_antigo().into_iter().find(|d| d.join("selecao.json").is_file() || d.join("registro.json").is_file())
    else {
        return;
    };
    let _ = std::fs::create_dir_all(&p.dados);
    for nome in ["selecao.json", "registro.json"] {
        let _ = std::fs::copy(antiga.join(nome), p.dados.join(nome));
    }
}

fn pastas(app: &tauri::AppHandle) -> Pastas {
    app.state::<EstadoBaixar>().pastas.clone()
}

async fn em_segundo_plano<T: Send + 'static>(trabalho: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(trabalho).await.map_err(|e| e.to_string())?
}

/// Tira do registro o que estava na pasta `pasta` (quando a publicação é
/// excluída pela Biblioteca), para a sincronização baixar de novo se ainda
/// estiver na seleção.
pub fn esquecer_pasta(app: &tauri::AppHandle, pasta: &str) {
    let p = pastas(app);
    if let Ok(mut r) = Registro::abrir(&p.registro()) {
        let antes = r.baixados.len();
        r.baixados.retain(|_, b| b.pasta != pasta);
        if r.baixados.len() != antes {
            let _ = r.salvar(&p.registro());
        }
    }
}

/// A capa de cada sigla_idioma que já está na Biblioteca (pasta → nome da imagem).
fn capas(p: &Pastas, registro: &Registro) -> BTreeMap<String, (String, String)> {
    let locais: BTreeMap<String, String> =
        leitor_nucleo::biblioteca::cartoes(&p.biblioteca).into_iter().filter_map(|c| Some((c.pasta.clone(), c.capa?))).collect();
    let mut saida = BTreeMap::new();
    for b in registro.baixados.values() {
        if let Some(c) = locais.get(&b.pasta) {
            saida.insert(format!("{}_{}", b.pedido.sigla, b.pedido.lingua), (b.pasta.clone(), c.clone()));
        }
    }
    saida
}

#[derive(Serialize)]
pub struct ItemDoCatalogo {
    sigla: &'static str,
    nome: &'static str,
    categoria: &'static str,
    nome_da_categoria: &'static str,
    periodico: bool,
}

#[derive(Serialize)]
pub struct Catalogo {
    itens: Vec<ItemDoCatalogo>,
    idiomas: Vec<(&'static str, &'static str)>,
    selecao: Vec<Escolha>,
    baixados: Vec<String>,
    /// sigla_idioma → (pasta, imagem da capa) na Biblioteca.
    capas: BTreeMap<String, (String, String)>,
}

#[tauri::command]
pub async fn baixar_catalogo(app: tauri::AppHandle) -> Result<Catalogo, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let selecao = Selecao::abrir(&p.selecao())?;
        let ingles = leitor_nucleo::idioma::ingles();
        Ok(Catalogo {
            itens: CATALOGO
                .iter()
                .map(|i| ItemDoCatalogo {
                    sigla: i.sigla,
                    nome: if ingles && !i.nome_en.is_empty() { i.nome_en } else { i.nome },
                    categoria: i.categoria,
                    nome_da_categoria: leitor_nucleo::biblioteca::nome_da_categoria(i.categoria),
                    periodico: i.periodico.is_some(),
                })
                .collect(),
            idiomas: IDIOMAS.to_vec(),
            selecao: selecao.escolhas,
            capas: capas(&p, &registro),
            baixados: registro.baixados.keys().cloned().collect(),
        })
    })
    .await
}

#[derive(Serialize)]
pub struct ItemDaSelecao {
    escolha: Escolha,
    nome: String,
    baixados: Vec<String>,
    capa: Option<(String, String)>,
}

#[tauri::command]
pub async fn baixar_selecao(app: tauri::AppHandle) -> Result<Vec<ItemDaSelecao>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let registro = Registro::abrir(&p.registro())?;
        let selecao = Selecao::abrir(&p.selecao())?;
        let capas = capas(&p, &registro);
        let ingles = leitor_nucleo::idioma::ingles();
        Ok(selecao
            .escolhas
            .into_iter()
            .map(|e| {
                let (sigla, lingua) = (e.pedido.sigla.clone(), e.pedido.lingua.clone());
                let deste: Vec<&leitor_baixar::Baixado> = registro
                    .baixados
                    .values()
                    .filter(|b| b.pedido.sigla == sigla && b.pedido.lingua == lingua)
                    .filter(|b| e.pedido.edicao.is_empty() || b.pedido.edicao == e.pedido.edicao)
                    .collect();
                let nome = catalogo::achar(&sigla)
                    .map(|i| if ingles && !i.nome_en.is_empty() { i.nome_en } else { i.nome }.to_string())
                    .or_else(|| deste.first().map(|b| b.arquivo.publicacao.clone()))
                    .unwrap_or_else(|| sigla.clone());
                ItemDaSelecao {
                    capa: capas.get(&format!("{sigla}_{lingua}")).cloned(),
                    baixados: deste.iter().map(|b| b.pedido.chave()).collect(),
                    nome,
                    escolha: e,
                }
            })
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn baixar_escolher(app: tauri::AppHandle, sigla: String, lingua: String, edicao: String, novas: bool) -> Result<bool, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let pedido = Pedido::novo(&sigla.to_lowercase(), &lingua, &edicao);
        if pedido.sigla.is_empty() || pedido.lingua.is_empty() {
            return Err(t("Falta a sigla ou o idioma.", "The code or the language is missing.").into());
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
pub async fn baixar_tirar(app: tauri::AppHandle, sigla: String, lingua: String, edicao: String) -> Result<bool, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let mut s = Selecao::abrir(&p.selecao())?;
        let tirou = s.tirar(&Pedido::novo(&sigla, &lingua, &edicao));
        s.salvar(&p.selecao())?;
        Ok(tirou)
    })
    .await
}

#[derive(Clone, Serialize)]
struct Progresso {
    chave: String,
    feito: u64,
    total: u64,
}

type Trabalho = Box<dyn FnOnce(&mut Cliente, &Pastas, &mut Registro, &mut dyn FnMut(&Pedido, u64, u64)) -> Vec<Desfecho> + Send>;

fn comecar(app: tauri::AppHandle, trabalho: Trabalho) -> Result<(), String> {
    let estado = app.state::<EstadoBaixar>();
    if estado.ocupado.swap(true, Ordering::SeqCst) {
        return Err(if estado.procurando.load(Ordering::SeqCst) {
            t("Espere a procura terminar.", "Wait for the check to finish.")
        } else {
            t("Já está baixando.", "Already downloading.")
        }
        .into());
    }
    let ocupado = estado.ocupado.clone();
    let p = estado.pastas.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let resultado = (|| -> Result<Vec<Desfecho>, String> {
            let mut registro = Registro::abrir(&p.registro())?;
            let estado = app.state::<EstadoBaixar>();
            let mut cliente = estado.cliente.lock().map_err(|e| e.to_string())?;
            let mut ultimo = (String::new(), u64::MAX);
            Ok(trabalho(&mut cliente, &p, &mut registro, &mut |pedido, feito, total| {
                let chave = pedido.chave();
                let pct = (feito * 100).checked_div(total).unwrap_or(0);
                if ultimo.0 != chave || ultimo.1 != pct {
                    ultimo = (chave.clone(), pct);
                    let _ = app.emit("baixar-progresso", Progresso { chave, feito, total });
                }
            }))
        })();
        ocupado.store(false, Ordering::SeqCst);
        let fim = match resultado {
            Ok(desfechos) => serde_json::json!({ "desfechos": desfechos }),
            Err(erro) => serde_json::json!({ "erro": erro }),
        };
        let _ = app.emit("baixar-fim", fim);
    });
    Ok(())
}

/// "Baixar agora": baixa o que falta e confere o que já está.
#[tauri::command]
pub fn baixar_sincronizar(app: tauri::AppHandle) -> Result<(), String> {
    comecar(
        app,
        Box::new(|cliente, p, registro, progresso| match Selecao::abrir(&p.selecao()) {
            Ok(selecao) => leitor_baixar::sincronizar(cliente, p, registro, &selecao, progresso),
            Err(e) => vec![Desfecho::Erro(Pedido::novo("selecao", "", ""), e)],
        }),
    )
}

/// Baixa o que a última procura achou.
#[tauri::command]
pub fn baixar_atualizar(app: tauri::AppHandle) -> Result<(), String> {
    let pendentes = std::mem::take(&mut *app.state::<EstadoBaixar>().pendentes.lock().map_err(|e| e.to_string())?);
    if pendentes.is_empty() {
        return Err(t("Nada para atualizar. Procure de novo.", "Nothing to update. Check again.").into());
    }
    comecar(app, Box::new(move |cliente, p, registro, progresso| leitor_baixar::atualizar(cliente, p, registro, &pendentes, progresso)))
}

#[tauri::command]
pub fn baixar_ocupado(app: tauri::AppHandle) -> bool {
    let e = app.state::<EstadoBaixar>();
    e.ocupado.load(Ordering::SeqCst) && !e.procurando.load(Ordering::SeqCst)
}

#[derive(Serialize)]
pub struct RespostaDaProcura {
    versoes: usize,
    novas: usize,
    novidades: Vec<Atualizacao>,
    erros: Vec<String>,
    sozinho: bool,
}

/// Procura atualizações. Sem `forcar` (ao abrir o app), só se passou mais
/// de 24 h da última e se há algo na seleção. `None`: não era hora.
#[tauri::command]
pub async fn baixar_procurar(app: tauri::AppHandle, forcar: bool) -> Result<Option<RespostaDaProcura>, String> {
    let p = pastas(&app);
    let registro = Registro::abrir(&p.registro())?;
    let selecao = Selecao::abrir(&p.selecao())?;
    if !forcar && (selecao.escolhas.is_empty() || !leitor_baixar::precisa_verificar(registro.ultima_verificacao, leitor_baixar::agora())) {
        return Ok(None);
    }
    let estado = app.state::<EstadoBaixar>();
    if estado.ocupado.swap(true, Ordering::SeqCst) {
        return if forcar { Err(t("Espere o download terminar.", "Wait for the download to finish.").into()) } else { Ok(None) };
    }
    estado.procurando.store(true, Ordering::SeqCst);
    let app2 = app.clone();
    let resultado = em_segundo_plano(move || {
        let estado = app2.state::<EstadoBaixar>();
        let procura = {
            let mut cliente = estado.cliente.lock().map_err(|e| e.to_string())?;
            leitor_baixar::procurar(&mut cliente, &registro, &selecao, catalogo::hoje())
        };
        if procura.valeu() {
            Registro::marcar_verificacao(&p.registro(), leitor_baixar::agora())?;
        }
        *estado.pendentes.lock().map_err(|e| e.to_string())? = procura.novidades.clone();
        let versoes = procura.novidades.iter().filter(|n| n.atual.is_some()).count();
        Ok(RespostaDaProcura {
            novas: procura.novidades.len() - versoes,
            versoes,
            novidades: procura.novidades,
            erros: procura.erros,
            sozinho: selecao.atualizar_sozinho,
        })
    })
    .await;
    estado.procurando.store(false, Ordering::SeqCst);
    estado.ocupado.store(false, Ordering::SeqCst);
    resultado.map(Some)
}

#[derive(Serialize)]
pub struct Opcoes {
    atualizar_sozinho: bool,
    ultima_verificacao: u64,
}

#[tauri::command]
pub async fn baixar_opcoes(app: tauri::AppHandle) -> Result<Opcoes, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        Ok(Opcoes {
            atualizar_sozinho: Selecao::abrir(&p.selecao())?.atualizar_sozinho,
            ultima_verificacao: Registro::abrir(&p.registro())?.ultima_verificacao,
        })
    })
    .await
}

#[tauri::command]
pub async fn baixar_atualizar_sozinho(app: tauri::AppHandle, valor: bool) -> Result<(), String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let mut s = Selecao::abrir(&p.selecao())?;
        s.atualizar_sozinho = valor;
        s.salvar(&p.selecao())
    })
    .await
}

// ---------------------------------------------------------------- catálogo completo

use leitor_baixar::catalogo_jw;

#[derive(Serialize)]
pub struct InfoDoCatalogo {
    versao: String,
    baixado_em: u64,
}

#[tauri::command]
pub fn catalogo_info(app: tauri::AppHandle) -> Option<InfoDoCatalogo> {
    catalogo_jw::versao_local(&pastas(&app).dados).map(|(versao, baixado_em)| InfoDoCatalogo { versao, baixado_em })
}

/// Baixa o catálogo do jw.org (se mudou). Andamento pelo evento `catalogo-progresso`.
#[tauri::command]
pub async fn catalogo_atualizar(app: tauri::AppHandle) -> Result<bool, String> {
    let p = pastas(&app);
    let app2 = app.clone();
    em_segundo_plano(move || {
        let mut ultimo = u64::MAX;
        catalogo_jw::atualizar(&p.dados, &mut |feito, total| {
            let pct = (feito * 100).checked_div(total).unwrap_or(0);
            if pct != ultimo {
                ultimo = pct;
                let _ = app2.emit("catalogo-progresso", (feito, total));
            }
        })
    })
    .await
}

fn meps(lingua: &str) -> Result<i64, String> {
    catalogo_jw::meps_de(lingua).ok_or_else(|| format!("{lingua}: idioma fora da lista"))
}

/// O que está instalado na Biblioteca, por (sigla, edição, língua) → pasta.
fn instaladas(biblioteca: &Path) -> BTreeMap<(String, i64, i64), leitor_nucleo::biblioteca::Cartao> {
    leitor_nucleo::biblioteca::cartoes(biblioteca).into_iter().map(|c| ((c.sigla.clone(), c.edicao, c.lingua), c)).collect()
}

#[derive(Serialize)]
pub struct CategoriaDoCatalogo {
    chave: &'static str,
    nome: &'static str,
    quantas: i64,
    instaladas: usize,
}

#[tauri::command]
pub async fn catalogo_categorias(app: tauri::AppHandle, lingua: String) -> Result<Vec<CategoriaDoCatalogo>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let m = meps(&lingua)?;
        let todas = instaladas(&p.biblioteca);
        let mut saida: Vec<CategoriaDoCatalogo> = catalogo_jw::categorias(&p.dados, m)?
            .into_iter()
            .map(|(chave, quantas)| CategoriaDoCatalogo {
                chave,
                nome: leitor_nucleo::biblioteca::nome_da_categoria(chave),
                quantas,
                instaladas: todas.values().filter(|c| c.lingua == m && c.categoria == chave).count(),
            })
            .collect();
        // Instaladas de categoria que o catálogo não tem nesta língua.
        for c in todas.values().filter(|c| c.lingua == m) {
            if !saida.iter().any(|x| x.chave == c.categoria) {
                let chave = catalogo_jw::ORDEM.iter().find(|k| **k == c.categoria).copied().unwrap_or("outras");
                saida.push(CategoriaDoCatalogo { chave, nome: leitor_nucleo::biblioteca::nome_da_categoria(chave), quantas: 0, instaladas: 1 });
            }
        }
        Ok(saida)
    })
    .await
}

#[tauri::command]
pub async fn catalogo_anos(app: tauri::AppHandle, lingua: String, categoria: String) -> Result<Vec<i64>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || catalogo_jw::anos(&p.dados, meps(&lingua)?, &categoria)).await
}

#[derive(Serialize)]
pub struct ItemDaCategoria {
    #[serde(flatten)]
    publicacao: catalogo_jw::PubDoCatalogo,
    /// A pasta na Biblioteca, se já está instalada.
    pasta: Option<String>,
    /// A capa da pasta instalada (melhor que a da internet sem conexão).
    capa_local: Option<String>,
    /// O jw.org tem versão mais nova que a instalada.
    atualizar: bool,
    chave: String,
}

#[tauri::command]
pub async fn catalogo_publicacoes(app: tauri::AppHandle, lingua: String, categoria: String, ano: Option<i64>) -> Result<Vec<ItemDaCategoria>, String> {
    let p = pastas(&app);
    em_segundo_plano(move || {
        let m = meps(&lingua)?;
        let todas = instaladas(&p.biblioteca);
        let registro = Registro::abrir(&p.registro()).unwrap_or_default();
        Ok(catalogo_jw::lista(&p.dados, m, &categoria, ano)?
            .into_iter()
            .map(|pubc| {
                let instalada = todas.get(&(pubc.simbolo.clone(), pubc.edicao, m));
                let chave = Pedido::novo(&pubc.simbolo, &lingua, &catalogo_jw::edicao_da_api(pubc.edicao)).chave();
                let atualizar = match (instalada, registro.baixados.get(&chave)) {
                    (Some(_), Some(b)) => !b.arquivo.modificado.is_empty() && b.arquivo.modificado < pubc.modificado,
                    _ => false,
                };
                ItemDaCategoria {
                    pasta: instalada.map(|c| c.pasta.clone()),
                    capa_local: instalada.and_then(|c| c.capa.clone()),
                    atualizar,
                    chave,
                    publicacao: pubc,
                }
            })
            .collect())
    })
    .await
}

/// Baixa uma publicação do catálogo (a nuvem do cartão). Andamento pelos
/// mesmos eventos da aba de seleção (`baixar-progresso` e `baixar-fim`).
#[tauri::command]
pub fn baixar_publicacao(app: tauri::AppHandle, simbolo: String, lingua: String, edicao: i64) -> Result<(), String> {
    let pedido = Pedido::novo(&simbolo, &lingua, &catalogo_jw::edicao_da_api(edicao));
    comecar(
        app,
        Box::new(move |cliente, p, registro, progresso| {
            vec![match leitor_baixar::baixar(cliente, p, registro, &pedido, &mut |f, t| progresso(&pedido, f, t)) {
                Ok(r) => Desfecho::Feito(r),
                Err(e) => Desfecho::Erro(pedido.clone(), e),
            }]
        }),
    )
}
