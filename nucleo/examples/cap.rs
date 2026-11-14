use std::path::Path;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let b = leitor_nucleo::biblia::Biblia::achar(Path::new(&a[1])).unwrap();
    println!("{}", b.capitulo(a[2].parse().unwrap(), a[3].parse().unwrap()).unwrap());
}
