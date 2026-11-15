//! A API pública de download do jw.org: `GETPUBMEDIALINKS`, a mesma que os
//! botões "Baixar" do site usam. Devolve, para cada arquivo, a URL, o tamanho,
//! o MD5 e a data de modificação, que é como o baixador confere o download e
//! percebe atualização.
//!
//! Boas maneiras: no máximo uma pergunta por segundo e um User-Agent que diz
//! o que é o programa. Nada de raspar páginas do site.
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

pub const API: &str = "https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS";
pub const AGENTE: &str = concat!(
    "JW-Library-on-Linux-Baixador/",
    env!("CARGO_PKG_VERSION"),
    " (app da comunidade, uso pessoal)"
);
const ESPERA: Duration = Duration::from_millis(1100);

/// O que se pede: a sigla (`nwtsty`, `w`, `mwb`, `lff`…), o código de idioma
/// do jw.org (`T` português, `E` inglês, `S` espanhol) e, nos periódicos, a
/// edição com seis dígitos (`202609`). Publicação sem edição leva `""`: a API
/// devolve lista vazia se receber o parâmetro sem precisar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pedido {
    pub sigla: String,
    pub lingua: String,
    #[serde(default)]
    pub edicao: String,
}

impl Pedido {
    pub fn novo(sigla: &str, lingua: &str, edicao: &str) -> Self {
        Pedido { sigla: sigla.trim().to_string(), lingua: lingua.trim().to_string(), edicao: edicao.trim().to_string() }
    }

    /// Chave do registro: `w_T_202609`, `nwtsty_T`.
    pub fn chave(&self) -> String {
        if self.edicao.is_empty() {
            format!("{}_{}", self.sigla, self.lingua)
        } else {
            format!("{}_{}_{}", self.sigla, self.lingua, self.edicao)
        }
    }
}

/// Um `.jwpub` oferecido pela API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arquivo {
    /// Título do arquivo (na Sentinela: o mês; na Bíblia: o nome).
    pub titulo: String,
    /// `pubName`: o nome da publicação.
    pub publicacao: String,
    pub url: String,
    pub md5: String,
    /// `modifiedDatetime`, como a API escreve ("2026-06-01 10:00:00").
    pub modificado: String,
    pub tamanho: u64,
    #[serde(default)]
    pub capa: Option<String>,
}

impl Arquivo {
    /// O nome do arquivo no servidor (`w_T_202609.jwpub`), sem pasta.
    pub fn nome(&self) -> Option<String> {
        let sem_consulta = self.url.split(['?', '#']).next().unwrap_or("");
        let nome = leitor_nucleo::jwpub::nome_seguro(sem_consulta)?;
        nome.to_ascii_lowercase().ends_with(".jwpub").then_some(nome)
    }

    /// Mudou desde o que foi baixado? O MD5 manda; a data só serve quando
    /// algum dos dois veio sem MD5.
    pub fn difere_de(&self, outro: &Arquivo) -> bool {
        if !self.md5.is_empty() && !outro.md5.is_empty() {
            return !self.md5.eq_ignore_ascii_case(&outro.md5);
        }
        self.modificado != outro.modificado || self.tamanho != outro.tamanho
    }
}

fn texto(v: &serde_json::Value, caminho: &str) -> String {
    v.pointer(caminho).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

/// Lê a resposta da API. `None` quando não há `.jwpub` nesse idioma.
pub fn ler_resposta(json: &str, lingua: &str) -> Result<Option<Arquivo>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("resposta da API: {e}"))?;
    let lista = v
        .get("files")
        .and_then(|f| f.get(lingua))
        .and_then(|l| l.get("JWPUB"))
        .and_then(|j| j.as_array());
    let Some(lista) = lista else { return Ok(None) };
    let publicacao = texto(&v, "/pubName");
    let capa = Some(texto(&v, "/pubImage/url")).filter(|c| !c.is_empty());
    for item in lista {
        let url = texto(item, "/file/url");
        if url.is_empty() {
            continue;
        }
        let arquivo = Arquivo {
            titulo: tirar_tags(&texto(item, "/title")),
            publicacao: tirar_tags(&publicacao),
            md5: texto(item, "/file/checksum").to_ascii_lowercase(),
            modificado: texto(item, "/file/modifiedDatetime"),
            tamanho: item.get("filesize").and_then(|t| t.as_u64()).unwrap_or(0),
            capa: capa.clone(),
            url,
        };
        if arquivo.nome().is_some() {
            return Ok(Some(arquivo));
        }
    }
    Ok(None)
}

fn tirar_tags(s: &str) -> String {
    let mut saida = String::with_capacity(s.len());
    let mut dentro = false;
    for c in s.chars() {
        match c {
            '<' => dentro = true,
            '>' => dentro = false,
            c if !dentro => saida.push(c),
            _ => {}
        }
    }
    // O título vem como HTML (ex.: "Revisão de&nbsp;2015"): desfaz as entidades
    // mais comuns; `&amp;` por último para não criar entidade nova.
    let saida = saida
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    saida.trim().to_string()
}

/// Copia `origem` para `destino` calculando o MD5 no caminho; confere tamanho
/// e MD5 no fim. O progresso recebe (bytes até agora, total esperado).
pub fn gravar_conferindo(
    origem: &mut dyn Read,
    destino: &Path,
    esperado: &Arquivo,
    progresso: &mut dyn FnMut(u64, u64),
) -> Result<(), String> {
    let mut saida = std::fs::File::create(destino).map_err(|e| format!("criar {}: {e}", destino.display()))?;
    let mut md5 = Md5::new();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = origem.read(&mut buffer).map_err(|e| format!("download interrompido: {e}"))?;
        if n == 0 {
            break;
        }
        md5.update(&buffer[..n]);
        saida.write_all(&buffer[..n]).map_err(|e| format!("gravar: {e}"))?;
        total += n as u64;
        progresso(total, esperado.tamanho);
    }
    saida.sync_all().map_err(|e| format!("gravar: {e}"))?;
    if esperado.tamanho > 0 && total != esperado.tamanho {
        return Err(format!("download incompleto: {total} de {} bytes", esperado.tamanho));
    }
    let calculado = format!("{:x}", md5.finalize());
    if !esperado.md5.is_empty() && !calculado.eq_ignore_ascii_case(&esperado.md5) {
        return Err("o arquivo chegou corrompido (MD5 diferente)".into());
    }
    Ok(())
}

/// Fala com a API e com o servidor de arquivos, respeitando a espera.
pub struct Cliente {
    agente: ureq::Agent,
    ultima: Option<Instant>,
}

impl Default for Cliente {
    fn default() -> Self {
        Self::novo()
    }
}

impl Cliente {
    pub fn novo() -> Self {
        let agente = ureq::AgentBuilder::new()
            .user_agent(AGENTE)
            .timeout_connect(Duration::from_secs(20))
            .timeout_read(Duration::from_secs(60))
            .build();
        Cliente { agente, ultima: None }
    }

    fn esperar(&mut self) {
        if let Some(ultima) = self.ultima {
            let passou = ultima.elapsed();
            if passou < ESPERA {
                sleep(ESPERA - passou);
            }
        }
        self.ultima = Some(Instant::now());
    }

    /// O que a API oferece para o pedido. `None`: não existe nesse idioma.
    pub fn consultar(&mut self, pedido: &Pedido) -> Result<Option<Arquivo>, String> {
        self.esperar();
        let mut req = self
            .agente
            .get(API)
            .query("output", "json")
            .query("pub", &pedido.sigla)
            .query("langwritten", &pedido.lingua)
            .query("fileformat", "JWPUB");
        if !pedido.edicao.is_empty() {
            req = req.query("issue", &pedido.edicao);
        }
        match req.call() {
            Ok(resp) => {
                let corpo = resp.into_string().map_err(|e| format!("ler resposta: {e}"))?;
                ler_resposta(&corpo, &pedido.lingua)
            }
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(ureq::Error::Status(codigo, _)) => Err(format!("a API respondeu {codigo}")),
            Err(e) => Err(format!("sem conexão com o jw.org: {e}")),
        }
    }

    /// Baixa para `pasta/<nome>.part` e troca para `<nome>` só no fim, já
    /// conferido. Devolve o caminho do `.jwpub`.
    pub fn baixar(&mut self, arquivo: &Arquivo, pasta: &Path, progresso: &mut dyn FnMut(u64, u64)) -> Result<PathBuf, String> {
        let nome = arquivo.nome().ok_or("URL sem nome de arquivo .jwpub")?;
        std::fs::create_dir_all(pasta).map_err(|e| format!("criar {}: {e}", pasta.display()))?;
        let final_ = pasta.join(&nome);
        let parcial = pasta.join(format!("{nome}.part"));
        self.esperar();
        let resp = match self.agente.get(&arquivo.url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(codigo, _)) => return Err(format!("o servidor respondeu {codigo}")),
            Err(e) => return Err(format!("sem conexão: {e}")),
        };
        let mut leitor = resp.into_reader();
        if let Err(e) = gravar_conferindo(&mut leitor, &parcial, arquivo, progresso) {
            let _ = std::fs::remove_file(&parcial);
            return Err(e);
        }
        std::fs::rename(&parcial, &final_).map_err(|e| format!("renomear: {e}"))?;
        Ok(final_)
    }
}
