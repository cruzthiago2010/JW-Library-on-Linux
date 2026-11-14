use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    for p in leitor_nucleo::jwpub::instaladas(Path::new(&a[1])) { println!("{} sigla={}", p.pasta, p.sigla); }
    let s = leitor_nucleo::reunioes::semana(Path::new(&a[1]), a[2].parse().unwrap());
    println!("{}", serde_json::to_string_pretty(&s).unwrap());
}
