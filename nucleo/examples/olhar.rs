// Uso: cargo run -p leitor-nucleo --example olhar -- <arquivo.jwpub> <pasta-de-instalação>
use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let raiz = Path::new(&a[2]);
    let nome = leitor_nucleo::jwpub::instalar(Path::new(&a[1]), raiz).unwrap();
    let p = leitor_nucleo::jwpub::Publicacao::abrir(raiz, &nome).unwrap();
    println!("{} | cat={} tipo={} sigla={} ano={} ed={}", p.titulo, p.categoria, p.tipo, p.sigla, p.ano, p.edicao);
    if let Some(n) = a.get(3) {
        let ids = p.documentos().unwrap();
        let d = p.documento(ids[n.parse::<usize>().unwrap()]).unwrap();
        println!("{}", d.html);
    }
}
