// Conta as classes CSS usadas em todos os documentos instalados (para escrever o CSS próprio).
// Uso: cargo run -p leitor-nucleo --example classes -- <pasta-de-instalação>
use std::collections::BTreeMap;
use std::path::Path;
fn main() {
    let raiz = std::env::args().nth(1).unwrap();
    let re = regex::Regex::new(r#"class="([^"]*)""#).unwrap();
    let mut conta: BTreeMap<String, usize> = BTreeMap::new();
    for p in leitor_nucleo::jwpub::instaladas(Path::new(&raiz)) {
        for id in p.documentos().unwrap_or_default() {
            if let Ok(d) = p.documento(id) {
                for c in re.captures_iter(&d.html) {
                    for t in c[1].split_whitespace() {
                        *conta.entry(t.to_string()).or_default() += 1;
                    }
                }
            }
        }
    }
    for (c, n) in conta { println!("{n}\t{c}"); }
}
