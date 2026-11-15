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
    /// O nome em inglês, para a interface em inglês.
    pub nome_en: &'static str,
    /// Chave da categoria em `leitor_nucleo::biblioteca::CATEGORIAS`.
    pub categoria: &'static str,
    pub periodico: Option<Periodicidade>,
}


pub const CATALOGO: &[Item] = &[
    Item { sigla: "nwtsty", nome: "Tradução do Novo Mundo (edição de estudo)", nome_en: "New World Translation (Study Edition)", categoria: "biblia", periodico: None },
    Item { sigla: "nwt", nome: "Tradução do Novo Mundo", nome_en: "New World Translation", categoria: "biblia", periodico: None },
    Item { sigla: "lff", nome: "Seja Feliz para Sempre!", nome_en: "Enjoy Life Forever!", categoria: "livros", periodico: None },
    Item { sigla: "lfb", nome: "Aprenda com as Histórias da Bíblia", nome_en: "Lessons You Can Learn From the Bible", categoria: "livros", periodico: None },
    Item { sigla: "bt", nome: "Testemunho Cabal sobre o Reino de Deus", nome_en: "“Bearing Thorough Witness” About God’s Kingdom", categoria: "livros", periodico: None },
    Item { sigla: "cl", nome: "Achegue-se a Jeová", nome_en: "Draw Close to Jehovah", categoria: "livros", periodico: None },
    Item { sigla: "ia", nome: "Imite a Sua Fé", nome_en: "Imitate Their Faith", categoria: "livros", periodico: None },
    Item { sigla: "jy", nome: "Jesus — o Caminho, a Verdade e a Vida", nome_en: "Jesus—The Way, the Truth, the Life", categoria: "livros", periodico: None },
    Item { sigla: "rr", nome: "A Adoração Pura de Jeová É Restaurada!", nome_en: "Pure Worship of Jehovah—Restored At Last!", categoria: "livros", periodico: None },
    Item { sigla: "kr", nome: "O Reino de Deus Já Governa!", nome_en: "God’s Kingdom Rules!", categoria: "livros", periodico: None },
    Item { sigla: "od", nome: "Organizados para Fazer a Vontade de Jeová", nome_en: "Organized to Do Jehovah’s Will", categoria: "livros", periodico: None },
    Item { sigla: "sjj", nome: "Cante de Coração para Jeová", nome_en: "Sing Out Joyfully to Jehovah", categoria: "livros", periodico: None },
    Item { sigla: "scl", nome: "Textos Bíblicos para a Vida Cristã", nome_en: "Scriptures for Christian Living", categoria: "livros", periodico: None },
    Item { sigla: "it", nome: "Estudo Perspicaz das Escrituras", nome_en: "Insight on the Scriptures", categoria: "indice", periodico: None },
    Item { sigla: "th", nome: "Melhore Sua Leitura e Seu Ensino", nome_en: "Apply Yourself to Reading and Teaching", categoria: "brochuras", periodico: None },
    Item { sigla: "lmd", nome: "Ame as Pessoas — Faça Discípulos", nome_en: "Love People—Make Disciples", categoria: "brochuras", periodico: None },
    Item { sigla: "ll", nome: "Escute a Deus e Viva para Sempre", nome_en: "Listen to God and Live Forever", categoria: "brochuras", periodico: None },
    Item { sigla: "fg", nome: "Boas Notícias de Deus para Você!", nome_en: "Good News From God!", categoria: "brochuras", periodico: None },
    Item { sigla: "w", nome: "A Sentinela (estudo)", nome_en: "The Watchtower (Study Edition)", categoria: "sentinela", periodico: Some(Periodicidade::Mensal) },
    Item { sigla: "mwb", nome: "Apostila da Reunião Vida e Ministério", nome_en: "Life and Ministry Meeting Workbook", categoria: "apostilas", periodico: Some(Periodicidade::Bimestral) },
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
