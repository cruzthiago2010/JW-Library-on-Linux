//! Testes com publicações de verdade, se houver: LEITOR_BIBLIOTECA=<pasta com as publicações instaladas>.
use std::path::PathBuf;

fn raiz() -> Option<PathBuf> {
    std::env::var("LEITOR_BIBLIOTECA").ok().map(PathBuf::from)
}

#[test]
fn semana_acha_apostila_e_sentinela() {
    let Some(raiz) = raiz() else { return };
    // 9-15 de novembro de 2026: apostila mwb_T_202611 e a Sentinela de setembro.
    let s = leitor_nucleo::reunioes::semana(&raiz, 20261111);
    let a = s.apostila.expect("apostila");
    assert_eq!(a.titulo, "9-15 de novembro");
    let w = s.sentinela.expect("sentinela");
    assert_eq!(w.inicio, 20261109);
    println!("sentinela: {} ({})", w.titulo, w.publicacao);
}

#[test]
fn painel_tem_notas() {
    let Some(raiz) = raiz() else { return };
    let p = leitor_nucleo::jwpub::Publicacao::abrir(&raiz, "w_T_202612").unwrap();
    let painel = leitor_nucleo::estudo::painel(&p, 5, "conteudo://localhost").unwrap();
    assert_eq!(painel.notas.len(), 4);
    assert!(painel.notas.iter().all(|n| !n.html.is_empty()));
    println!("trechos: {}", painel.trechos.len());
}
