//! A "minha seleção": o que a pessoa quer ter. Publicação comum fica em dia
//! (baixa e atualiza); periódico com `novas_edicoes` baixa sozinho cada
//! edição nova que aparecer.
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::api::Pedido;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Escolha {
    pub pedido: Pedido,
    /// Só nos periódicos sem edição fixa: baixar cada edição nova.
    #[serde(default)]
    pub novas_edicoes: bool,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Selecao {
    #[serde(default)]
    pub escolhas: Vec<Escolha>,
}

impl Selecao {
    pub fn abrir(caminho: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(caminho) {
            Ok(t) => serde_json::from_str(&t).map_err(|e| format!("seleção {}: {e}", caminho.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Selecao::default()),
            Err(e) => Err(format!("ler {}: {e}", caminho.display())),
        }
    }

    pub fn salvar(&self, caminho: &Path) -> Result<(), String> {
        if let Some(pai) = caminho.parent() {
            std::fs::create_dir_all(pai).map_err(|e| format!("criar {}: {e}", pai.display()))?;
        }
        let tmp = caminho.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).map_err(|e| e.to_string())?)
            .map_err(|e| format!("gravar {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, caminho).map_err(|e| format!("gravar {}: {e}", caminho.display()))
    }

    /// Põe ou troca (pela chave do pedido). Devolve `false` se já estava igual.
    pub fn escolher(&mut self, escolha: Escolha) -> bool {
        match self.escolhas.iter_mut().find(|e| e.pedido == escolha.pedido) {
            Some(e) if *e == escolha => false,
            Some(e) => {
                *e = escolha;
                true
            }
            None => {
                self.escolhas.push(escolha);
                self.escolhas.sort_by(|a, b| a.pedido.cmp(&b.pedido));
                true
            }
        }
    }

    pub fn tirar(&mut self, pedido: &Pedido) -> bool {
        let antes = self.escolhas.len();
        self.escolhas.retain(|e| &e.pedido != pedido);
        self.escolhas.len() != antes
    }
}
