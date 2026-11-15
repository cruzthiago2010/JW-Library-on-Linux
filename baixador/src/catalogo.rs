//! O catálogo para escolher: uma lista embutida, por categoria (as mesmas da
//! Biblioteca do leitor), com a sigla que a API do jw.org entende. Não há
//! API pública que liste tudo, e raspar o site está fora; por isso a lista é
//! nossa, e quem quiser outra publicação digita a sigla.
//!
//! `jw-baixador conferir-catalogo T` pergunta à API por cada item (uma vez por
//! segundo) e diz quais não existem no idioma.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Periodicidade {
    /// Edição todo mês (A Sentinela de estudo).
    Mensal,
    /// Edição nos meses ímpares (a apostila).
    Bimestral,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub sigla: &'static str,
    pub nome: &'static str,
    /// Chave da categoria em `leitor_nucleo::biblioteca::CATEGORIAS`.
    pub categoria: &'static str,
    pub periodico: Option<Periodicidade>,
}

const fn item(sigla: &'static str, nome: &'static str, categoria: &'static str) -> Item {
    Item { sigla, nome, categoria, periodico: None }
}

pub const CATALOGO: &[Item] = &[
    item("nwtsty", "Tradução do Novo Mundo (edição de estudo)", "biblia"),
    item("nwt", "Tradução do Novo Mundo", "biblia"),
    item("lff", "Seja Feliz para Sempre!", "livros"),
    item("lfb", "Aprenda com as Histórias da Bíblia", "livros"),
    item("bt", "Testemunho Cabal sobre o Reino de Deus", "livros"),
    item("cl", "Achegue-se a Jeová", "livros"),
    item("ia", "Imite a Sua Fé", "livros"),
    item("jy", "Jesus — o Caminho, a Verdade e a Vida", "livros"),
    item("rr", "A Adoração Pura de Jeová É Restaurada!", "livros"),
    item("kr", "O Reino de Deus Já Governa!", "livros"),
    item("od", "Organizados para Fazer a Vontade de Jeová", "livros"),
    item("sjj", "Cante de Coração para Jeová", "livros"),
    item("scl", "Textos Bíblicos para a Vida Cristã", "livros"),
    item("it", "Estudo Perspicaz das Escrituras", "indice"),
    item("th", "Melhore Sua Leitura e Seu Ensino", "brochuras"),
    item("lmd", "Ame as Pessoas — Faça Discípulos", "brochuras"),
    item("ll", "Escute a Deus e Viva para Sempre", "brochuras"),
    item("fg", "Boas Notícias de Deus para Você!", "brochuras"),
    Item { sigla: "w", nome: "A Sentinela (estudo)", categoria: "sentinela", periodico: Some(Periodicidade::Mensal) },
    Item { sigla: "mwb", nome: "Apostila da Reunião Vida e Ministério", categoria: "apostilas", periodico: Some(Periodicidade::Bimestral) },
];

/// Idiomas mais pedidos, pelo código do jw.org. Outros podem ser digitados.
pub const IDIOMAS: &[(&str, &str)] = &[
    ("T", "Português (Brasil)"),
    ("TPO", "Português (Portugal)"),
    ("E", "English"),
    ("S", "Español"),
    ("F", "Français"),
    ("X", "Deutsch"),
    ("I", "Italiano"),
];

pub fn achar(sigla: &str) -> Option<&'static Item> {
    CATALOGO.iter().find(|i| i.sigla == sigla)
}

/// As edições de um periódico perto de uma data (ano, mês): de dois meses
/// antes a três depois, porque a edição sai antes das semanas que serve.
/// Na bimestral, só os meses ímpares.
pub fn edicoes_perto(p: Periodicidade, ano: i32, mes: u32) -> Vec<String> {
    let base = ano * 12 + mes as i32 - 1;
    (-2..=3)
        .map(|d| base + d)
        .map(|m| (m / 12, (m % 12) as u32 + 1))
        .filter(|(_, m)| p == Periodicidade::Mensal || m % 2 == 1)
        .map(|(a, m)| format!("{a}{m:02}"))
        .collect()
}

/// (ano, mês) de hoje, em UTC, sem dependência de calendário.
pub fn hoje() -> (i32, u32) {
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    ano_mes(segundos.div_euclid(86_400))
}

/// Dias desde 1970-01-01 para (ano, mês) — algoritmo "civil from days".
pub fn ano_mes(dias: i64) -> (i32, u32) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let ano = (yoe + era * 400 + if mes <= 2 { 1 } else { 0 }) as i32;
    (ano, mes)
}
