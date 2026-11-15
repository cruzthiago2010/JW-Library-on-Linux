//! O que já foi baixado: um JSON pequeno na pasta do baixador, com o MD5 e a
//! data de cada arquivo, para saber quando há versão nova.
use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::api::{Arquivo, Pedido};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Baixado {
    pub pedido: Pedido,
    pub arquivo: Arquivo,
    /// A pasta da publicação dentro da Biblioteca do leitor.
    pub pasta: String,
    /// Segundos desde 1970.
    pub baixado_em: u64,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Registro {
    #[serde(default)]
    pub baixados: BTreeMap<String, Baixado>,
    /// Quando foi a última procura por atualizações (segundos desde 1970;
    /// 0 = nunca). Vale para a janela e para o timer: no máximo uma por dia.
    #[serde(default)]
    pub ultima_verificacao: u64,
}

/// De quanto em quanto tempo procurar atualizações sozinho.
pub const INTERVALO: u64 = 24 * 60 * 60;

/// A regra das 24 h: procura de novo quando passou um dia desde a última,
/// ou quando o relógio voltou para trás (a última ficou "no futuro").
pub fn precisa_verificar(ultima: u64, agora: u64) -> bool {
    ultima > agora || agora - ultima >= INTERVALO
}

impl Registro {
    /// Abre o registro; se não existe (primeira vez), começa vazio.
    pub fn abrir(caminho: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(caminho) {
            Ok(texto) => serde_json::from_str(&texto).map_err(|e| format!("registro {}: {e}", caminho.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Registro::default()),
            Err(e) => Err(format!("ler {}: {e}", caminho.display())),
        }
    }

    /// Anota a hora da última verificação. Relê o registro antes, para não
    /// desfazer o que outro (a janela, o timer) gravou enquanto se procurava.
    pub fn marcar_verificacao(caminho: &Path, quando: u64) -> Result<(), String> {
        let mut r = Registro::abrir(caminho)?;
        r.ultima_verificacao = quando;
        r.salvar(caminho)
    }

    /// Grava num arquivo temporário e troca, para nunca ficar pela metade.
    pub fn salvar(&self, caminho: &Path) -> Result<(), String> {
        if let Some(pai) = caminho.parent() {
            std::fs::create_dir_all(pai).map_err(|e| format!("criar {}: {e}", pai.display()))?;
        }
        let tmp = caminho.with_extension("json.tmp");
        let texto = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, texto).map_err(|e| format!("gravar {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, caminho).map_err(|e| format!("gravar {}: {e}", caminho.display()))
    }
}
