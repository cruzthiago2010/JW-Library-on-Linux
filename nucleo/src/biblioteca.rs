//! A Biblioteca: as publicações instaladas, agrupadas nas categorias do
//! JW Library (Livros, Brochuras, A Sentinela…), com capa e ano.
use std::path::Path;

use crate::jwpub::{self, Publicacao};

/// Categorias na ordem da tela PUBLICAÇÕES do JW Library.
pub const CATEGORIAS: &[(&str, &str, &[&str])] = &[
    ("biblia", "Bíblias", &["bi"]),
    ("livros", "Livros", &["bk"]),
    ("brochuras", "Brochuras e Livretos", &["brch", "bklt"]),
    ("folhetos", "Folhetos e Convites", &["tr", "trct", "ivt"]),
    ("series", "Séries de artigos", &["ser"]),
    ("sentinela", "A Sentinela", &["w"]),
    ("despertai", "Despertai!", &["g"]),
    ("apostilas", "Apostilas", &["mwb"]),
    ("ministerio", "Ministério do Reino", &["km"]),
    ("programas", "Programas", &["pgm", "prg"]),
    ("indice", "Índice", &["dx", "it"]),
    ("orientacoes", "Orientações", &["manual", "gd"]),
];

/// A chave da categoria de uma publicação; o que não se conhece vai para "outras".
pub fn categoria_de(p: &Publicacao) -> &'static str {
    let simbolo = p.categoria.as_str();
    for (chave, _, simbolos) in CATEGORIAS {
        if simbolos.contains(&simbolo) {
            return chave;
        }
    }
    // .jwpub antigo sem a coluna: pela sigla.
    match p.sigla.as_str() {
        "w" | "wp" => "sentinela",
        "g" => "despertai",
        "mwb" => "apostilas",
        s if s.starts_with("nwt") => "biblia",
        _ => "outras",
    }
}

pub fn nome_da_categoria(chave: &str) -> &'static str {
    if crate::idioma::ingles() {
        return match chave {
            "biblia" => "Bibles",
            "livros" => "Books",
            "brochuras" => "Brochures and Booklets",
            "folhetos" => "Tracts and Invitations",
            "series" => "Article Series",
            "sentinela" => "The Watchtower",
            "despertai" => "Awake!",
            "apostilas" => "Meeting Workbooks",
            "ministerio" => "Kingdom Ministry",
            "programas" => "Programs",
            "indice" => "Indexes",
            "orientacoes" => "Guidelines",
            _ => "Other",
        };
    }
    CATEGORIAS.iter().find(|c| c.0 == chave).map(|c| c.1).unwrap_or("Outras")
}

#[derive(serde::Serialize, Clone)]
pub struct Cartao {
    pub pasta: String,
    pub titulo: String,
    pub curto: String,
    pub simbolo: String,
    pub ano: i64,
    pub edicao: i64,
    pub categoria: String,
    pub capa: Option<String>,
    /// Bytes ocupados no disco (banco + imagens).
    pub tamanho: u64,
}

#[derive(serde::Serialize)]
pub struct Categoria {
    pub chave: String,
    pub nome: String,
    pub quantas: usize,
}

fn tamanho_da_pasta(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|d| {
            d.filter_map(Result::ok)
                .map(|e| {
                    let m = e.metadata().ok();
                    match m {
                        Some(m) if m.is_dir() => tamanho_da_pasta(&e.path()),
                        Some(m) => m.len(),
                        None => 0,
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Todas as publicações instaladas como cartões.
pub fn cartoes(raiz: &Path) -> Vec<Cartao> {
    jwpub::instaladas(raiz)
        .into_iter()
        .map(|p| Cartao {
            capa: p.capa(),
            tamanho: tamanho_da_pasta(&raiz.join(&p.pasta)),
            categoria: categoria_de(&p).to_string(),
            pasta: p.pasta,
            titulo: p.titulo,
            curto: p.curto,
            simbolo: p.simbolo,
            ano: p.ano,
            edicao: p.edicao,
        })
        .collect()
}

/// As categorias que têm alguma publicação, na ordem do JW Library.
pub fn categorias(cartoes: &[Cartao]) -> Vec<Categoria> {
    let mut chaves: Vec<&str> = CATEGORIAS.iter().map(|c| c.0).collect();
    chaves.push("outras");
    chaves
        .into_iter()
        .filter_map(|chave| {
            let quantas = cartoes.iter().filter(|c| c.categoria == chave).count();
            (quantas > 0).then(|| Categoria {
                chave: chave.to_string(),
                nome: nome_da_categoria(chave).to_string(),
                quantas,
            })
        })
        .collect()
}
