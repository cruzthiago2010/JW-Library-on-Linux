//! Baixar `.jwpub` do jw.org para a Biblioteca do JW Library on Linux.
//! Veio do projeto "Baixador para JW Library on Linux" (agora embutido no leitor):
//! catálogo, seleção, registro do que foi baixado e procura de atualizações.
//!
//! Pergunta à API pública do jw.org, baixa o arquivo conferindo o MD5 e o
//! instala direto na Biblioteca do leitor com a mesma função do botão
//! Importar (`leitor_nucleo::jwpub::instalar`). O `.jwpub` baixado é apagado
//! depois de instalado: a Biblioteca guarda o banco e as imagens, e o
//! registro guarda o MD5 para perceber atualização.
pub mod api;
pub mod catalogo;
pub mod catalogo_jw;
pub mod registro;
pub mod selecao;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use api::{Arquivo, Cliente, Pedido};
pub use registro::{precisa_verificar, Baixado, Registro, INTERVALO};
pub use selecao::{Escolha, Selecao};

/// As pastas que o baixador usa.
#[derive(Debug, Clone)]
pub struct Pastas {
    /// A Biblioteca do JW Library on Linux.
    pub biblioteca: PathBuf,
    /// A pasta do baixador (registro e downloads em andamento).
    pub dados: PathBuf,
}

impl Pastas {
    pub fn registro(&self) -> PathBuf {
        self.dados.join("registro.json")
    }

    pub fn selecao(&self) -> PathBuf {
        self.dados.join("selecao.json")
    }

    pub fn partes(&self) -> PathBuf {
        self.dados.join("partes")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Situacao {
    /// Primeira vez.
    Novo,
    /// Havia versão anterior e chegou uma nova.
    Atualizado,
    /// Já estava baixado e igual ao do servidor.
    JaEstava,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Resultado {
    pub situacao: Situacao,
    pub pasta: String,
    pub arquivo: Arquivo,
}

pub fn agora() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn instalada(biblioteca: &Path, pasta: &str) -> bool {
    biblioteca.join(pasta).join("instalado.json").is_file()
}

/// Instala um `.jwpub` já baixado e anota no registro. `forcar` apaga a marca
/// de instalação antes, porque o leitor pula a instalação quando o tamanho é
/// o mesmo, e uma correção pode ter o mesmo tamanho.
pub fn instalar_baixado(
    pastas: &Pastas,
    registro: &mut Registro,
    pedido: &Pedido,
    arquivo: &Arquivo,
    jwpub: &Path,
    forcar: bool,
) -> Result<String, String> {
    std::fs::create_dir_all(&pastas.biblioteca).map_err(|e| format!("criar a Biblioteca: {e}"))?;
    if forcar {
        if let Some(pasta) = jwpub.file_stem().and_then(|s| s.to_str()) {
            let _ = std::fs::remove_file(pastas.biblioteca.join(pasta).join("instalado.json"));
        }
    }
    let pasta = leitor_nucleo::jwpub::instalar(jwpub, &pastas.biblioteca)?;
    registro.baixados.insert(
        pedido.chave(),
        Baixado { pedido: pedido.clone(), arquivo: arquivo.clone(), pasta: pasta.clone(), baixado_em: agora() },
    );
    registro.salvar(&pastas.registro())?;
    Ok(pasta)
}

/// Pergunta, baixa (se preciso) e instala. Não baixa de novo o que já está
/// instalado e igual ao do servidor.
pub fn baixar(
    cliente: &mut Cliente,
    pastas: &Pastas,
    registro: &mut Registro,
    pedido: &Pedido,
    progresso: &mut dyn FnMut(u64, u64),
) -> Result<Resultado, String> {
    let arquivo = cliente
        .consultar(pedido)?
        .ok_or_else(|| format!("{} não tem .jwpub nesse idioma (ou a sigla/edição não existe)", pedido.chave()))?;
    trazer(cliente, pastas, registro, pedido, arquivo, progresso)
}

/// Como `baixar`, para quem já consultou a API.
pub fn trazer(
    cliente: &mut Cliente,
    pastas: &Pastas,
    registro: &mut Registro,
    pedido: &Pedido,
    arquivo: Arquivo,
    progresso: &mut dyn FnMut(u64, u64),
) -> Result<Resultado, String> {
    let anterior = registro.baixados.get(&pedido.chave()).cloned();
    if let Some(a) = &anterior {
        if !arquivo.difere_de(&a.arquivo) && instalada(&pastas.biblioteca, &a.pasta) {
            return Ok(Resultado { situacao: Situacao::JaEstava, pasta: a.pasta.clone(), arquivo });
        }
    }
    let jwpub = cliente.baixar(&arquivo, &pastas.partes(), progresso)?;
    let atualizado = anterior.is_some();
    let instalado = instalar_baixado(pastas, registro, pedido, &arquivo, &jwpub, atualizado);
    let _ = std::fs::remove_file(&jwpub);
    let pasta = instalado?;
    let situacao = if atualizado { Situacao::Atualizado } else { Situacao::Novo };
    Ok(Resultado { situacao, pasta, arquivo })
}

/// Algo para baixar que a procura achou: versão nova de um baixado
/// (`atual` preenchido) ou edição/publicação que ainda não está aqui.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Atualizacao {
    pub pedido: Pedido,
    pub atual: Option<Arquivo>,
    pub novo: Arquivo,
}

/// O que conferir (numa sincronização ou numa procura): os pedidos fixos da
/// seleção, sempre, e, para cada periódico com `novas_edicoes`, só as
/// edições perto de hoje — baixadas ou não; as baixadas são conferidas pelo
/// MD5. Edições antigas não entram, para não multiplicar as perguntas.
pub fn pedidos_da_sincronizacao(selecao: &Selecao, _registro: &Registro, hoje: (i32, u32)) -> Vec<Pedido> {
    let mut pedidos: Vec<Pedido> = vec![];
    for e in &selecao.escolhas {
        let periodico = catalogo::achar(&e.pedido.sigla).and_then(|i| i.periodico);
        let novos: Vec<Pedido> = match periodico {
            Some(p) if e.novas_edicoes && e.pedido.edicao.is_empty() => catalogo::edicoes_perto(p, hoje.0, hoje.1)
                .iter()
                .map(|edicao| Pedido::novo(&e.pedido.sigla, &e.pedido.lingua, edicao))
                .collect(),
            _ => vec![e.pedido.clone()],
        };
        for p in novos {
            // Uma edição fixa que também cai na janela das edições novas: uma vez só.
            if !pedidos.contains(&p) {
                pedidos.push(p);
            }
        }
    }
    pedidos
}

/// É edição de um periódico marcado com "edições novas"? (Então "não
/// existe" quer dizer só que ainda não saiu.)
fn edicao_automatica(selecao: &Selecao, pedido: &Pedido) -> bool {
    selecao
        .escolhas
        .iter()
        .any(|e| e.novas_edicoes && e.pedido.sigla == pedido.sigla && e.pedido.lingua == pedido.lingua && e.pedido.edicao.is_empty())
}

/// O resultado de uma procura.
#[derive(Debug, Default, serde::Serialize)]
pub struct Procura {
    pub novidades: Vec<Atualizacao>,
    pub erros: Vec<String>,
    /// Quantos pedidos a API respondeu (0 com tudo dando erro: sem internet).
    pub respondidos: usize,
}

impl Procura {
    /// Conta como verificação feita? Só se o jw.org respondeu (ou não havia
    /// nada para perguntar); sem internet, tenta de novo na próxima vez.
    pub fn valeu(&self) -> bool {
        self.respondidos > 0 || self.erros.is_empty()
    }
}

/// Procura atualizações sem baixar nada: pergunta à API (uma por segundo)
/// pelo mesmo conjunto da sincronização e devolve o que mudou ou falta.
/// Erros de um item não param os outros.
pub fn procurar(cliente: &mut Cliente, registro: &Registro, selecao: &Selecao, hoje: (i32, u32)) -> Procura {
    let mut saida = Procura::default();
    for pedido in pedidos_da_sincronizacao(selecao, registro, hoje) {
        match cliente.consultar(&pedido) {
            Ok(Some(novo)) => {
                saida.respondidos += 1;
                let atual = registro.baixados.get(&pedido.chave()).map(|b| b.arquivo.clone());
                if atual.as_ref().is_none_or(|a| novo.difere_de(a)) {
                    saida.novidades.push(Atualizacao { pedido, atual, novo });
                }
            }
            Ok(None) => {
                saida.respondidos += 1;
                if !edicao_automatica(selecao, &pedido) {
                    saida.erros.push(format!("{}: não existe nesse idioma", pedido.chave()));
                }
            }
            Err(e) => saida.erros.push(format!("{}: {e}", pedido.chave())),
        }
    }
    saida
}

/// Baixa o que uma procura achou, sem perguntar à API de novo.
pub fn atualizar(
    cliente: &mut Cliente,
    pastas: &Pastas,
    registro: &mut Registro,
    novidades: &[Atualizacao],
    progresso: &mut dyn FnMut(&Pedido, u64, u64),
) -> Vec<Desfecho> {
    novidades
        .iter()
        .map(|n| {
            match trazer(cliente, pastas, registro, &n.pedido, n.novo.clone(), &mut |f, t| progresso(&n.pedido, f, t)) {
                Ok(r) => Desfecho::Feito(r),
                Err(e) => Desfecho::Erro(n.pedido.clone(), e),
            }
        })
        .collect()
}

/// "2 publicações têm versão nova", "1 edição nova para baixar"… O texto do
/// aviso na janela e da notificação. `None`: nada novo.
pub fn resumo(novidades: &[Atualizacao]) -> Option<String> {
    let versoes = novidades.iter().filter(|n| n.atual.is_some()).count();
    let novas = novidades.len() - versoes;
    let parte_versoes = match versoes {
        0 => None,
        1 => Some("1 publicação tem versão nova".to_string()),
        n => Some(format!("{n} publicações têm versão nova")),
    };
    let parte_novas = match novas {
        0 => None,
        1 => Some("1 publicação nova para baixar".to_string()),
        n => Some(format!("{n} publicações novas para baixar")),
    };
    match (parte_versoes, parte_novas) {
        (Some(a), Some(b)) => Some(format!("{a} e {b}")),
        (a, b) => a.or(b),
    }
}

/// O que aconteceu com cada pedido de uma sincronização.
#[derive(Debug, serde::Serialize)]
pub enum Desfecho {
    Feito(Resultado),
    /// Edição que ainda não saiu (ou não existe no idioma).
    NaoSaiu(Pedido),
    Erro(Pedido, String),
}

/// Põe a Biblioteca em dia com a seleção: baixa o que falta, atualiza o que
/// mudou (também nas edições recentes já baixadas) e traz as edições novas
/// dos periódicos marcados. Conta como verificação (anota a hora).
pub fn sincronizar(
    cliente: &mut Cliente,
    pastas: &Pastas,
    registro: &mut Registro,
    selecao: &Selecao,
    progresso: &mut dyn FnMut(&Pedido, u64, u64),
) -> Vec<Desfecho> {
    let pedidos = pedidos_da_sincronizacao(selecao, registro, catalogo::hoje());
    let mut saida = vec![];
    let mut respondidos = 0;
    for pedido in pedidos {
        let consulta = cliente.consultar(&pedido);
        if consulta.is_ok() {
            respondidos += 1;
        }
        match consulta {
            Ok(None) if edicao_automatica(selecao, &pedido) => saida.push(Desfecho::NaoSaiu(pedido)),
            Ok(None) => saida.push(Desfecho::Erro(pedido, "não existe nesse idioma".into())),
            Ok(Some(arquivo)) => match trazer(cliente, pastas, registro, &pedido, arquivo, &mut |f, t| progresso(&pedido, f, t)) {
                Ok(r) => saida.push(Desfecho::Feito(r)),
                Err(e) => saida.push(Desfecho::Erro(pedido, e)),
            },
            Err(e) => saida.push(Desfecho::Erro(pedido, e)),
        }
    }
    if respondidos > 0 || saida.is_empty() {
        registro.ultima_verificacao = agora();
        let _ = Registro::marcar_verificacao(&pastas.registro(), registro.ultima_verificacao);
    }
    saida
}
