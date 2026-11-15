//! A Bíblia, lida do `.jwpub` da Bíblia (de preferência a de Estudo, `nwtsty`):
//! o mesmo texto e a mesma marcação que o JW Library mostra. Copiado do app
//! Reuniões nativo.
//!
//! - `BibleChapter.Content`: o capítulo inteiro (cifrado como os documentos);
//! - `BibleVerse`: um versículo por linha, com numeração global — o versículo
//!   `n` do capítulo é `FirstVerseId + n - 1`.
//!
//! Códigos de versículo: `BBCCCVVV` (livro, capítulo, versículo).

use rusqlite::OptionalExtension;

use crate::jwpub::{instaladas, Publicacao, Resultado};

/// As siglas da grade e o tom de cada livro: os mesmos de
/// `app/services/biblia_nav.py` (capturas do JW Library, 25/09/2026).
pub const SIGLAS: [&str; 66] = [
    "Gên", "Êx", "Le", "Núm", "De", "Jos", "Jz", "Ru", "1Sa", "2Sa", "1Rs", "2Rs",
    "1Cr", "2Cr", "Esd", "Ne", "Est", "Jó", "Sal", "Pr", "Ec", "Cân", "Is", "Je",
    "La", "Ez", "Da", "Os", "Jl", "Am", "Ob", "Jon", "Miq", "Na", "Hab", "Sof",
    "Ag", "Za", "Mal",
    "Mt", "Mr", "Lu", "Jo", "At", "Ro", "1Co", "2Co", "Gál", "Ef", "Fil", "Col",
    "1Te", "2Te", "1Ti", "2Ti", "Tit", "Flm", "He", "Tg", "1Pe", "2Pe", "1Jo",
    "2Jo", "3Jo", "Ju", "Ap",
];

/// Os nomes dos livros, como o título do capítulo no JW Library ("João 3").
pub const NOMES: [&str; 66] = [
    "Gênesis", "Êxodo", "Levítico", "Números", "Deuteronômio", "Josué", "Juízes", "Rute",
    "1 Samuel", "2 Samuel", "1 Reis", "2 Reis", "1 Crônicas", "2 Crônicas", "Esdras",
    "Neemias", "Ester", "Jó", "Salmos", "Provérbios", "Eclesiastes", "Cântico de Salomão",
    "Isaías", "Jeremias", "Lamentações", "Ezequiel", "Daniel", "Oseias", "Joel", "Amós",
    "Obadias", "Jonas", "Miqueias", "Naum", "Habacuque", "Sofonias", "Ageu", "Zacarias",
    "Malaquias", "Mateus", "Marcos", "Lucas", "João", "Atos", "Romanos", "1 Coríntios",
    "2 Coríntios", "Gálatas", "Efésios", "Filipenses", "Colossenses", "1 Tessalonicenses",
    "2 Tessalonicenses", "1 Timóteo", "2 Timóteo", "Tito", "Filêmon", "Hebreus", "Tiago",
    "1 Pedro", "2 Pedro", "1 João", "2 João", "3 João", "Judas", "Apocalipse",
];

/// Os nomes em inglês, na ordem de `NOMES`.
pub const NOMES_EN: [&str; 66] = [
    "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy", "Joshua", "Judges", "Ruth",
    "1 Samuel", "2 Samuel", "1 Kings", "2 Kings", "1 Chronicles", "2 Chronicles", "Ezra",
    "Nehemiah", "Esther", "Job", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon",
    "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel", "Amos",
    "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah", "Haggai", "Zechariah",
    "Malachi", "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "1 Corinthians",
    "2 Corinthians", "Galatians", "Ephesians", "Philippians", "Colossians", "1 Thessalonians",
    "2 Thessalonians", "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews", "James",
    "1 Peter", "2 Peter", "1 John", "2 John", "3 John", "Jude", "Revelation",
];

/// O nome do livro (1 a 66) no idioma da interface.
pub fn nome(livro: u32) -> &'static str {
    let lista = if crate::idioma::ingles() { &NOMES_EN } else { &NOMES };
    lista.get((livro as usize).wrapping_sub(1)).copied().unwrap_or("?")
}

fn tom(numero: u32) -> &'static str {
    match numero {
        1..=5 => "escuro",
        6..=17 => "claro",
        18..=22 => "medio",
        23..=43 => "escuro",
        44 => "claro",
        45..=65 => "medio",
        _ => "escuro",
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Livro {
    pub numero: u32,
    pub sigla: &'static str,
    pub nome: &'static str,
    pub capitulos: u32,
    pub tom: &'static str,
    /// Como no site: livro de um capítulo só abre direto no texto.
    pub url: String,
}

#[derive(serde::Serialize)]
pub struct Versiculo {
    pub livro: u32,
    pub capitulo: u32,
    pub verso: u32,
    /// O número como o arquivo marca (`<span class="vl">16</span>`; o 1 vem
    /// como `cl`, o número do capítulo).
    pub rotulo: String,
    pub html: String,
}

/// Um código `BBCCCVVV` → (livro, capítulo, versículo).
pub fn do_codigo(codigo: u32) -> (u32, u32, u32) {
    (codigo / 1_000_000, (codigo / 1_000) % 1_000, codigo % 1_000)
}

/// "43003016-43003018" (ou um código só) → os dois códigos, em ordem.
pub fn faixa(texto: &str) -> Option<(u32, u32)> {
    let mut partes = texto.splitn(2, '-');
    let inicio: u32 = partes.next()?.trim().parse().ok()?;
    let fim: u32 = match partes.next() {
        Some(f) => f.trim().parse().ok()?,
        None => inicio,
    };
    let (b, c, v) = do_codigo(inicio);
    ((1..=66).contains(&b) && c >= 1 && v >= 1 && fim >= inicio).then_some((inicio, fim))
}

/// "João 3:16", "João 3:16-18", "Gênesis 4:23–5:2", "Salmo 23" (capítulo inteiro).
pub fn rotulo(inicio: u32, fim: u32) -> String {
    let (b1, c1, v1) = do_codigo(inicio);
    let (b2, c2, v2) = do_codigo(fim);
    let nome1 = if b1 == 19 && c1 == c2 && b1 == b2 { crate::idioma::t("Salmo", "Psalm") } else { nome(b1) };
    if b1 != b2 {
        format!("{} {c1}:{v1}–{} {c2}:{v2}", nome(b1), nome(b2))
    } else if c1 != c2 {
        format!("{nome1} {c1}:{v1}–{c2}:{v2}")
    } else if v1 == v2 {
        format!("{nome1} {c1}:{v1}")
    } else {
        format!("{nome1} {c1}:{v1}-{v2}")
    }
}

pub struct Biblia {
    pub publicacao: Publicacao,
}

impl Biblia {
    /// A Bíblia instalada (o primeiro `.jwpub` com símbolo `nwt…`).
    pub fn achar(raiz: &std::path::Path) -> Option<Self> {
        let mut todas: Vec<Publicacao> = instaladas(raiz).into_iter().filter(|p| p.simbolo.starts_with("nwt")).collect();
        // A de estudo primeiro (é a que tem as notas e o conteúdo de cada livro).
        todas.sort_by_key(|p| p.simbolo != "nwtsty");
        todas.into_iter().next().map(|publicacao| Self { publicacao })
    }

    pub fn livros(&self) -> Resultado<Vec<Livro>> {
        let conexao = self.publicacao.banco_publico()?;
        let mut consulta = conexao
            .prepare("SELECT BookNumber, MAX(ChapterNumber) FROM BibleChapter GROUP BY BookNumber")
            .map_err(|e| e.to_string())?;
        let capitulos: std::collections::HashMap<u32, u32> = consulta
            .query_map([], |l| Ok((l.get(0)?, l.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        Ok((1..=66u32)
            .map(|numero| {
                let total = capitulos.get(&numero).copied().unwrap_or(1);
                Livro {
                    numero,
                    sigla: SIGLAS[numero as usize - 1],
                    nome: nome(numero),
                    capitulos: total,
                    tom: tom(numero),
                    url: if total == 1 { format!("/biblia/{numero}/1") } else { format!("/biblia/{numero}") },
                }
            })
            .collect())
    }

    /// O HTML do capítulo inteiro (o de `BibleChapter`).
    pub fn capitulo(&self, livro: u32, capitulo: u32) -> Resultado<String> {
        let conexao = self.publicacao.banco_publico()?;
        let bruto: Vec<u8> = conexao
            .query_row(
                "SELECT Content FROM BibleChapter WHERE BookNumber = ?1 AND ChapterNumber = ?2",
                [livro, capitulo],
                |l| l.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("{} {capitulo}: {}", nome(livro), crate::idioma::t("não está na Bíblia instalada", "not in the installed Bible")))?;
        self.publicacao.claro(&bruto)
    }

    /// O código `BBCCCVVV` de um versículo pela numeração global do arquivo.
    fn codigo_do_id(&self, conexao: &rusqlite::Connection, id: i64) -> Option<u32> {
        conexao
            .query_row(
                "SELECT BookNumber, ChapterNumber, FirstVerseId FROM BibleChapter \
                 WHERE ?1 BETWEEN FirstVerseId AND LastVerseId",
                [id],
                |l| Ok((l.get::<_, u32>(0)?, l.get::<_, u32>(1)?, l.get::<_, i64>(2)?)),
            )
            .optional()
            .ok()
            .flatten()
            .map(|(b, c, primeiro)| b * 1_000_000 + c * 1_000 + (id - primeiro + 1) as u32)
    }

    /// As referências cruzadas de uma letra do capítulo (`<span class="m"
    /// data-mid="78">a</span>`): as faixas de códigos que ela aponta, da tabela
    /// `BibleCitation` (BlockNumber = data-mid, dentro das citações do capítulo).
    pub fn referencias(&self, livro: u32, capitulo: u32, mid: i64) -> Resultado<Vec<(u32, u32)>> {
        let conexao = self.publicacao.banco_publico()?;
        let mut consulta = conexao
            .prepare(
                "SELECT c.FirstBibleVerseId, c.LastBibleVerseId FROM BibleCitation c \
                 JOIN BibleChapter b ON c.BibleCitationId BETWEEN b.FirstBibleCitationId AND b.LastBibleCitationId \
                 WHERE b.BookNumber = ?1 AND b.ChapterNumber = ?2 AND c.BlockNumber = ?3 \
                 ORDER BY c.ElementNumber, c.BibleCitationId",
            )
            .map_err(|e| e.to_string())?;
        let ids: Vec<(i64, i64)> = consulta
            .query_map(rusqlite::params![livro, capitulo, mid], |l| Ok((l.get(0)?, l.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        Ok(ids
            .into_iter()
            .filter_map(|(a, b)| Some((self.codigo_do_id(&conexao, a)?, self.codigo_do_id(&conexao, b)?)))
            .collect())
    }

    /// O HTML de cada versículo recortado do CAPÍTULO (que traz as letras das
    /// referências cruzadas, `<span class="m" data-mid>`; o de `BibleVerse`
    /// não traz), sem o número do versículo, que a folha desenha.
    pub fn versiculos_com_letras(&self, inicio: u32, fim: u32) -> Resultado<Vec<Versiculo>> {
        let mut saida = self.versiculos(inicio, fim)?;
        let mut capitulo: Option<((u32, u32), String)> = None;
        for v in saida.iter_mut() {
            if capitulo.as_ref().map(|c| c.0) != Some((v.livro, v.capitulo)) {
                capitulo = Some(((v.livro, v.capitulo), self.capitulo(v.livro, v.capitulo).unwrap_or_default()));
            }
            let html = &capitulo.as_ref().unwrap().1;
            let partes = recortar_versiculo(html, v.livro, v.capitulo, v.verso);
            if !partes.is_empty() {
                v.html = partes;
            }
        }
        Ok(saida)
    }

    /// Os versículos de uma faixa de códigos, atravessando capítulos e livros.
    pub fn versiculos(&self, inicio: u32, fim: u32) -> Resultado<Vec<Versiculo>> {
        let conexao = self.publicacao.banco_publico()?;
        let mut consulta = conexao
            .prepare(
                "SELECT BookNumber, ChapterNumber, FirstVerseId, LastVerseId FROM BibleChapter \
                 WHERE BookNumber * 1000 + ChapterNumber BETWEEN ?1 AND ?2 \
                 ORDER BY BookNumber, ChapterNumber",
            )
            .map_err(|e| e.to_string())?;
        let capitulos: Vec<(u32, u32, i64, i64)> = consulta
            .query_map([inicio / 1000, fim / 1000], |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?, l.get(3)?)))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        let mut verso = conexao
            .prepare("SELECT Label, Content FROM BibleVerse WHERE BibleVerseId = ?1")
            .map_err(|e| e.to_string())?;
        let mut saida = vec![];
        for (b, c, primeiro, ultimo) in capitulos {
            let total = (ultimo - primeiro + 1).max(0) as u32;
            for v in 1..=total {
                let codigo = b * 1_000_000 + c * 1_000 + v;
                if codigo < inicio || codigo > fim {
                    continue;
                }
                let linha: Option<(Option<String>, Vec<u8>)> = verso
                    .query_row([primeiro + v as i64 - 1], |l| Ok((l.get(0)?, l.get(1)?)))
                    .optional()
                    .map_err(|e| e.to_string())?;
                if let Some((rotulo, bruto)) = linha {
                    let rotulo = if v == 1 { r#"<span class="vl">1</span>"#.to_string() } else { rotulo.unwrap_or_default() };
                    saida.push(Versiculo { livro: b, capitulo: c, verso: v, rotulo, html: self.publicacao.claro(&bruto)? });
                }
            }
        }
        if saida.is_empty() {
            return Err(format!("{}: {}", rotulo(inicio, fim), crate::idioma::t("não está na Bíblia instalada", "not in the installed Bible")));
        }
        Ok(saida)
    }
}

#[derive(serde::Serialize)]
pub struct NotaDeEstudo {
    /// O versículo (1, 2…) a que a nota pertence.
    pub verso: u32,
    pub rotulo: String,
    pub html: String,
}

#[derive(serde::Serialize)]
pub struct Referencia {
    pub inicio: u32,
    pub fim: u32,
    pub rotulo: String,
    pub html: String,
}

/// Um link de texto bíblico do HTML (`NWTR/24:23:4-24:23:6`, `nwtsty/43:3:16`)
/// → a faixa de códigos.
pub fn faixa_do_link(link: &str) -> Option<(u32, u32)> {
    let corpo = link.rsplit('/').next()?;
    let mut partes = corpo.splitn(2, '-');
    let codigo = |t: &str| -> Option<u32> {
        let mut n = t.split(':').map(|x| x.trim().parse::<u32>().ok());
        let (b, c, v) = (n.next()??, n.next()??, n.next().flatten().unwrap_or(1));
        ((1..=66).contains(&b)).then_some(b * 1_000_000 + c * 1_000 + v)
    };
    let inicio = codigo(partes.next()?)?;
    let fim = match partes.next() {
        Some(f) => codigo(f)?,
        None => inicio,
    };
    (fim >= inicio).then_some((inicio, fim))
}

impl Biblia {
    /// As notas de estudo do capítulo (tabela `VerseCommentary`), por versículo.
    pub fn notas_de_estudo(&self, livro: u32, capitulo: u32, base: &str) -> Resultado<Vec<NotaDeEstudo>> {
        let conexao = self.publicacao.banco_publico()?;
        let Ok(mut consulta) = conexao.prepare(
            "SELECT m.BibleVerseId - b.FirstVerseId + 1, IFNULL(vc.Label, ''), vc.Content \
               FROM BibleChapter b \
               JOIN VerseCommentaryMap m ON m.BibleVerseId BETWEEN b.FirstVerseId AND b.LastVerseId \
               JOIN VerseCommentary vc ON vc.VerseCommentaryId = m.VerseCommentaryId \
              WHERE b.BookNumber = ?1 AND b.ChapterNumber = ?2 \
              ORDER BY m.BibleVerseId, vc.VerseCommentaryId",
        ) else {
            return Ok(vec![]); // Bíblia sem notas de estudo (nwt comum)
        };
        let linhas: Vec<(u32, String, Option<Vec<u8>>)> = consulta
            .query_map([livro, capitulo], |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?)))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        Ok(linhas
            .into_iter()
            .filter_map(|(verso, rotulo, bruto)| {
                let html = self.publicacao.claro(&bruto?).ok()?;
                Some(NotaDeEstudo {
                    verso,
                    rotulo: crate::leitor::ajustar_html(&rotulo, &self.publicacao.pasta, base),
                    html: crate::leitor::ajustar_html(&html, &self.publicacao.pasta, base),
                })
            })
            .collect())
    }

    /// Os versículos de uma faixa, como um bloco para o painel ou a folha.
    pub fn referencia(&self, inicio: u32, fim: u32) -> Resultado<Referencia> {
        let versos = self.versiculos(inicio, fim)?;
        let html = versos
            .iter()
            .map(|v| format!(r#"<span class="vnum">{}</span> {}"#, v.verso, v.html))
            .collect::<Vec<_>>()
            .join(" ");
        Ok(Referencia { inicio, fim, rotulo: rotulo(inicio, fim), html })
    }

    /// As referências marginais de uma letra, já com o texto de cada versículo.
    pub fn referencias_com_texto(&self, livro: u32, capitulo: u32, mid: i64) -> Resultado<Vec<Referencia>> {
        Ok(self
            .referencias(livro, capitulo, mid)?
            .into_iter()
            .filter_map(|(a, b)| self.referencia(a, b).ok())
            .collect())
    }
}

/// Os pedaços `<span id="vL-C-V-N" class="v">…</span>` de um versículo no
/// HTML do capítulo, sem o número (`class="vl"`/`"cl"`), juntos.
pub fn recortar_versiculo(html: &str, livro: u32, capitulo: u32, verso: u32) -> String {
    let marca = format!(r#"id="v{livro}-{capitulo}-{verso}-"#);
    let mut saida = String::new();
    let mut de = 0;
    while let Some(achou) = html[de..].find(&marca) {
        let abre = html[..de + achou].rfind("<span").unwrap_or(de + achou);
        let inicio_miolo = match html[abre..].find('>') { Some(i) => abre + i + 1, None => break };
        // Acha o </span> que fecha este, contando os <span> de dentro.
        let (mut nivel, mut i) = (1, inicio_miolo);
        while nivel > 0 && i < html.len() {
            let prox_abre = html[i..].find("<span").map(|x| i + x);
            let prox_fecha = match html[i..].find("</span>") { Some(x) => i + x, None => break };
            match prox_abre {
                Some(a) if a < prox_fecha => { nivel += 1; i = a + 5; }
                _ => { nivel -= 1; i = prox_fecha + 7; }
            }
        }
        let miolo = &html[inicio_miolo..i.saturating_sub(7).max(inicio_miolo)];
        saida.push_str(&tirar_numero(miolo));
        saida.push(' ');
        de = i;
    }
    saida.trim().to_string()
}

/// Tira o `<span … class="vl">2 …</span>` / `class="cl"` (o número do versículo
/// ou do capítulo) do começo do miolo.
fn tirar_numero(miolo: &str) -> String {
    static NUMERO: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = NUMERO.get_or_init(|| {
        regex::Regex::new(r#"(?s)^\s*<span data-vlid="[^"]*" class="(?:vl|cl)">.*?<span class="tt [^"]*"[^>]*></span></span>\s*"#).unwrap()
    });
    re.replace(miolo, "").into_owned()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn recorta_o_versiculo_com_as_letras() {
        let cap = concat!(
            r#"<p><span id="v43-3-1-1" class="v"><span data-vlid="vl0" class="cl"><strong>3</strong> <span class="tt cl" data-rel-vlid="vl0"></span></span>Entre os fariseus<span data-mid="78" class="m">a<span class="tt m" data-rel-mid="78"></span></span> era</span>"#,
            r#"<span id="v43-3-2-1" class="v"><span data-vlid="2" class="vl">2 <span class="tt vl" data-rel-vlid="2"></span></span>Ele veio</span></p>"#,
        );
        let v1 = recortar_versiculo(cap, 43, 3, 1);
        assert!(v1.starts_with("Entre os fariseus<span data-mid=\"78\" class=\"m\">a"), "{v1}");
        assert!(v1.ends_with("era"));
        assert_eq!(recortar_versiculo(cap, 43, 3, 2), "Ele veio");
        assert_eq!(recortar_versiculo(cap, 43, 3, 9), "");
    }

    #[test]
    fn link_de_texto_biblico() {
        assert_eq!(faixa_do_link("NWTR/24:23:4-24:23:4"), Some((24023004, 24023004)));
        assert_eq!(faixa_do_link("NWTR/43:10:2-43:10:5"), Some((43010002, 43010005)));
        assert_eq!(faixa_do_link("nwtsty/19:23"), Some((19023001, 19023001)));
        assert_eq!(faixa_do_link("lixo"), None);
    }

    #[test]
    fn faixas_e_rotulos() {
        assert_eq!(faixa("43003016"), Some((43003016, 43003016)));
        assert_eq!(faixa("43003016-43003018"), Some((43003016, 43003018)));
        assert_eq!(faixa("67001001"), None);
        assert_eq!(faixa("43003018-43003016"), None);
        assert_eq!(rotulo(43003016, 43003016), "João 3:16");
        assert_eq!(rotulo(43003016, 43003018), "João 3:16-18");
        assert_eq!(rotulo(1004023, 1005002), "Gênesis 4:23–5:2");
        assert_eq!(rotulo(19023001, 19023006), "Salmo 23:1-6");
    }
}
