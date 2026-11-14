// Texto coberto por cada grifo da Bíblia num backup real. Uso: conferir_biblia <biblioteca> <userData.db>
use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let b = leitor_nucleo::biblia::Biblia::achar(Path::new(&a[1])).unwrap();
    let u = rusqlite::Connection::open(&a[2]).unwrap();
    let mut s = u.prepare("select l.BookNumber,l.ChapterNumber,b.Identifier,b.StartToken,b.EndToken from BlockRange b join UserMark m using(UserMarkId) join Location l using(LocationId) where l.BookNumber is not null and l.KeySymbol='nwtsty' order by 1,2,3 limit 15").unwrap();
    for r in s.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, u32>(1)?, r.get::<_, u32>(2)?, r.get::<_, usize>(3)?, r.get::<_, usize>(4)?))).unwrap() {
        let (l, c, v, i, f) = r.unwrap();
        let cap = b.capitulo(l, c).unwrap();
        let html = leitor_nucleo::biblia::recortar_versiculo(&cap, l, c, v);
        let texto = leitor_nucleo::tokens::texto_do_paragrafo(&html);
        println!("{l}:{c}:{v} {i}-{f} ({} tokens): {}", leitor_nucleo::tokens::tokenizar(&texto).len(), leitor_nucleo::tokens::trecho(&texto, i, f).unwrap_or("??".into()));
    }
}
