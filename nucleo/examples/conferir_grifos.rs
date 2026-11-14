// Mostra o texto que cada grifo de um backup real cobre, usando a nossa regra de tokens.
// Uso: conferir_grifos <biblioteca> <userData.db> <pasta> <meps>
use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let p = leitor_nucleo::jwpub::Publicacao::abrir(Path::new(&a[1]), &a[3]).unwrap();
    let meps: i64 = a[4].parse().unwrap();
    let d = p.documento(p.pelo_meps(meps).unwrap()).unwrap();
    let re = regex::Regex::new(r#"(?s)<p[^>]*\bdata-pid="(\d+)"[^>]*>(.*?)</p>"#).unwrap();
    let paragrafos: std::collections::HashMap<i64, String> =
        re.captures_iter(&d.html).map(|c| (c[1].parse().unwrap(), leitor_nucleo::tokens::texto_do_paragrafo(&c[2]))).collect();
    let u = rusqlite::Connection::open(&a[2]).unwrap();
    let mut s = u.prepare("select b.Identifier,b.StartToken,b.EndToken from BlockRange b join UserMark m using(UserMarkId) join Location l using(LocationId) where l.DocumentId=? order by 1,2").unwrap();
    for r in s.query_map([meps], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))).unwrap() {
        let (pid, i, f) = r.unwrap();
        let t = paragrafos.get(&pid).and_then(|x| leitor_nucleo::tokens::trecho(x, i as usize, f as usize));
        println!("p{pid} {i}-{f}: {}", t.unwrap_or("??".into()));
    }
}
