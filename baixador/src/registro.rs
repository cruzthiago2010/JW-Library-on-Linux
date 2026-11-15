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
