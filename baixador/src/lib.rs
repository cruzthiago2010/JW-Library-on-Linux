//! Baixador de `.jwpub` para o JW Library on Linux.
//!
//! Pergunta à API pública do jw.org, baixa o arquivo conferindo o MD5 e o
//! instala direto na Biblioteca do leitor com a mesma função do botão
//! Importar (`leitor_nucleo::jwpub::instalar`). O `.jwpub` baixado é apagado
//! depois de instalado: a Biblioteca guarda o banco e as imagens, e o
//! registro guarda o MD5 para perceber atualização.
pub mod api;
pub mod catalogo;
#[cfg(feature = "janela")]
pub mod janela;
pub mod registro;
pub mod selecao;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use api::{Arquivo, Cliente, Pedido};
pub use registro::{Baixado, Registro};
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
    /// As de verdade: `~/.local/share/io.github.leitorjw/biblioteca` (a mesma
    /// que o Tauri dá ao leitor) e `~/.local/share/io.github.leitorjw.baixador`.
    /// `BAIXADOR_BIBLIOTECA` e `BAIXADOR_DADOS` trocam as duas, para testes.
    pub fn padrao() -> Result<Self, String> {
        let base = dirs::data_dir().ok_or("não achei a pasta de dados do usuário")?;
        let biblioteca = std::env::var_os("BAIXADOR_BIBLIOTECA")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("io.github.leitorjw").join("biblioteca"));
        let dados = std::env::var_os("BAIXADOR_DADOS")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("io.github.leitorjw.baixador"));
        Ok(Pastas { biblioteca, dados })
    }

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

fn agora() -> u64 {
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

/// Uma atualização disponível para algo já baixado.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Atualizacao {
    pub pedido: Pedido,
    pub atual: Arquivo,
    pub novo: Arquivo,
}

/// Pergunta à API por cada item do registro (uma pergunta por segundo) e
/// devolve os que mudaram. Erros de um item não param os outros.
pub fn verificar(cliente: &mut Cliente, registro: &Registro) -> (Vec<Atualizacao>, Vec<String>) {
    let mut novas = vec![];
    let mut erros = vec![];
    for b in registro.baixados.values() {
        match cliente.consultar(&b.pedido) {
            Ok(Some(novo)) if novo.difere_de(&b.arquivo) => {
                novas.push(Atualizacao { pedido: b.pedido.clone(), atual: b.arquivo.clone(), novo })
            }
            Ok(_) => {}
            Err(e) => erros.push(format!("{}: {e}", b.pedido.chave())),
        }
    }
    (novas, erros)
}

/// O que entra numa sincronização: os pedidos fixos da seleção e, para cada
/// periódico com `novas_edicoes`, as edições perto de hoje que ainda não
/// foram baixadas.
pub fn pedidos_da_sincronizacao(selecao: &Selecao, registro: &Registro, hoje: (i32, u32)) -> Vec<Pedido> {
    let mut pedidos = vec![];
    for e in &selecao.escolhas {
        let periodico = catalogo::achar(&e.pedido.sigla).and_then(|i| i.periodico);
        match periodico {
            Some(p) if e.novas_edicoes && e.pedido.edicao.is_empty() => {
                for edicao in catalogo::edicoes_perto(p, hoje.0, hoje.1) {
                    let pedido = Pedido::novo(&e.pedido.sigla, &e.pedido.lingua, &edicao);
                    if !registro.baixados.contains_key(&pedido.chave()) {
                        pedidos.push(pedido);
                    }
                }
            }
            _ => pedidos.push(e.pedido.clone()),
        }
    }
    pedidos
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
/// mudou e traz as edições novas dos periódicos marcados.
pub fn sincronizar(
    cliente: &mut Cliente,
    pastas: &Pastas,
    registro: &mut Registro,
    selecao: &Selecao,
    progresso: &mut dyn FnMut(&Pedido, u64, u64),
) -> Vec<Desfecho> {
    let pedidos = pedidos_da_sincronizacao(selecao, registro, catalogo::hoje());
    let mut saida = vec![];
    for pedido in pedidos {
        let periodico_sem_edicao_fixa = selecao
            .escolhas
            .iter()
            .any(|e| e.novas_edicoes && e.pedido.sigla == pedido.sigla && e.pedido.lingua == pedido.lingua && e.pedido.edicao.is_empty());
        match cliente.consultar(&pedido) {
            Ok(None) if periodico_sem_edicao_fixa => saida.push(Desfecho::NaoSaiu(pedido)),
            Ok(None) => saida.push(Desfecho::Erro(pedido, "não existe nesse idioma".into())),
            Ok(Some(arquivo)) => match trazer(cliente, pastas, registro, &pedido, arquivo, &mut |f, t| progresso(&pedido, f, t)) {
                Ok(r) => saida.push(Desfecho::Feito(r)),
                Err(e) => saida.push(Desfecho::Erro(pedido, e)),
            },
            Err(e) => saida.push(Desfecho::Erro(pedido, e)),
        }
    }
    saida
}
