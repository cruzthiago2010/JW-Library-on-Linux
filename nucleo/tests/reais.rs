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

#[test]
fn biblia_notas_referencias_e_busca() {
    let Some(raiz) = raiz() else { return };
    let b = leitor_nucleo::biblia::Biblia::achar(&raiz).expect("bíblia");
    let notas = b.notas_de_estudo(43, 3, "conteudo://localhost").unwrap();
    assert!(notas.iter().any(|n| n.verso == 16), "nota de estudo em João 3:16");
    let refs = b.referencias_com_texto(43, 3, 78).unwrap();
    assert!(!refs.is_empty());
    let dir = std::env::temp_dir().join("leitor-busca-teste.db");
    let (total, achados) = leitor_nucleo::busca::buscar(&raiz, &dir, "nascer de novo", true, 10).unwrap();
    assert!(total >= 1 && achados.iter().any(|a| a.codigo / 1000 == 43003), "{total}");
    let _ = std::fs::remove_file(dir);
}

#[test]
fn restaura_backup_real_do_aparelho() {
    let Ok(arquivo) = std::env::var("LEITOR_BACKUP") else { return };
    let saida = std::env::temp_dir().join("leitor-restaurado.db");
    let r = leitor_nucleo::backup::extrair(std::path::Path::new(&arquivo), &saida).unwrap();
    assert!(r.grifos > 0);
    println!("{r:?}");
    let _ = std::fs::remove_file(saida);
}

#[test]
fn busca_nas_publicacoes() {
    let Some(raiz) = raiz() else { return };
    let indice = std::env::temp_dir().join(format!("leitor-busca-pub-{}.db", std::process::id()));
    let (total, achados) = leitor_nucleo::busca::buscar_publicacoes(&raiz, &indice, "gibeonitas", false, 10).unwrap();
    assert!(total > 0);
    println!("{total} achados; 1º: {} — {} — {}", achados[0].titulo, achados[0].publicacao, achados[0].trecho);
    let _ = std::fs::remove_file(indice);
}
