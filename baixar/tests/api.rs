use std::io::Cursor;

use leitor_baixar::api::{gravar_conferindo, ler_resposta, Arquivo, Pedido};
use leitor_baixar::{Baixado, Registro};

const EXEMPLO: &str = include_str!("dados/exemplo_w_T.json");

#[test]
fn le_a_resposta_da_api() {
    let a = ler_resposta(EXEMPLO, "T").unwrap().expect("tem jwpub");
    assert_eq!(a.publicacao, "A Sentinela (Estudo)");
    assert_eq!(a.titulo, "Setembro de 2026");
    assert_eq!(a.nome().as_deref(), Some("w_T_202609.jwpub"));
    assert_eq!(a.md5, "5d41402abc4b2a76b9719d911017c592");
    assert_eq!(a.tamanho, 5);
    assert_eq!(a.modificado, "2026-06-02 08:15:00");
    assert_eq!(a.capa.as_deref(), Some("https://exemplo.invalid/w_T_202609_lsr.jpg"));
}

#[test]
fn outro_idioma_ou_resposta_vazia_e_none() {
    assert!(ler_resposta(EXEMPLO, "E").unwrap().is_none());
    assert!(ler_resposta("{}", "T").unwrap().is_none());
    assert!(ler_resposta("não é json", "T").is_err());
}

#[test]
fn chave_do_pedido() {
    assert_eq!(Pedido::novo("w", "T", "202609").chave(), "w_T_202609");
    assert_eq!(Pedido::novo(" nwtsty ", "T", "").chave(), "nwtsty_T");
}

fn arquivo() -> Arquivo {
    ler_resposta(EXEMPLO, "T").unwrap().unwrap()
}

#[test]
fn confere_o_md5_e_o_tamanho() {
    let pasta = std::env::temp_dir().join(format!("baixador-teste-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let destino = pasta.join("x.part");
    let mut visto = 0;
    // md5("hello") = 5d41402abc4b2a76b9719d911017c592
    gravar_conferindo(&mut Cursor::new(b"hello".to_vec()), &destino, &arquivo(), &mut |f, _| visto = f).unwrap();
    assert_eq!(visto, 5);
    assert_eq!(std::fs::read(&destino).unwrap(), b"hello");

    let erro = gravar_conferindo(&mut Cursor::new(b"hellp".to_vec()), &destino, &arquivo(), &mut |_, _| {}).unwrap_err();
    assert!(erro.contains("MD5"), "{erro}");
    let erro = gravar_conferindo(&mut Cursor::new(b"hel".to_vec()), &destino, &arquivo(), &mut |_, _| {}).unwrap_err();
    assert!(erro.contains("incompleto"), "{erro}");
    std::fs::remove_dir_all(&pasta).unwrap();
}

#[test]
fn percebe_versao_nova() {
    let a = arquivo();
    let mut b = a.clone();
    assert!(!a.difere_de(&b));
    b.md5 = "outro".into();
    assert!(a.difere_de(&b));
    let mut c = a.clone();
    c.modificado = "2026-07-01 00:00:00".into();
    assert!(!a.difere_de(&c), "com MD5 igual, a data não importa");
    let (mut d, mut e) = (a.clone(), a.clone());
    d.md5.clear();
    e.md5.clear();
    e.modificado = "2026-07-01 00:00:00".into();
    assert!(d.difere_de(&e), "sem MD5, vale a data");
}

#[test]
fn registro_vai_e_volta() {
    let pasta = std::env::temp_dir().join(format!("baixador-registro-{}", std::process::id()));
    let caminho = pasta.join("registro.json");
    assert_eq!(Registro::abrir(&caminho).unwrap(), Registro::default());
    let mut r = Registro::default();
    let p = Pedido::novo("w", "T", "202609");
    r.baixados.insert(p.chave(), Baixado { pedido: p, arquivo: arquivo(), pasta: "w_T_202609".into(), baixado_em: 1 });
    r.salvar(&caminho).unwrap();
    assert_eq!(Registro::abrir(&caminho).unwrap(), r);
    std::fs::remove_dir_all(&pasta).unwrap();
}

#[test]
fn edicoes_perto_de_hoje() {
    use leitor_baixar::catalogo::{ano_mes, edicoes_perto, Periodicidade};
    assert_eq!(ano_mes(0), (1970, 1));
    assert_eq!(ano_mes(20_772), (2026, 11)); // 15/11/2026
    assert_eq!(edicoes_perto(Periodicidade::Mensal, 2026, 11), ["202609", "202610", "202611", "202612", "202701", "202702"]);
    assert_eq!(edicoes_perto(Periodicidade::Bimestral, 2026, 11), ["202609", "202611", "202701"]);
    assert_eq!(edicoes_perto(Periodicidade::Bimestral, 2026, 1), ["202511", "202601", "202603"]);
}

#[test]
fn selecao_e_pedidos_da_sincronizacao() {
    use leitor_baixar::{pedidos_da_sincronizacao, Escolha, Selecao};
    let mut s = Selecao::default();
    assert!(s.escolher(Escolha { pedido: Pedido::novo("nwtsty", "T", ""), novas_edicoes: false }));
    assert!(s.escolher(Escolha { pedido: Pedido::novo("mwb", "T", ""), novas_edicoes: true }));
    assert!(!s.escolher(Escolha { pedido: Pedido::novo("mwb", "T", ""), novas_edicoes: true }));
    let mut r = Registro::default();
    let ja = Pedido::novo("mwb", "T", "202609");
    r.baixados.insert(ja.chave(), Baixado { pedido: ja, arquivo: arquivo(), pasta: "mwb_T_202609".into(), baixado_em: 1 });
    let chaves: Vec<String> = pedidos_da_sincronizacao(&s, &r, (2026, 11)).iter().map(|p| p.chave()).collect();
    // A edição já baixada (202609) também entra: é conferida pelo MD5.
    assert_eq!(chaves, ["mwb_T_202609", "mwb_T_202611", "mwb_T_202701", "nwtsty_T"]);
    assert!(s.tirar(&Pedido::novo("mwb", "T", "")));
    assert!(!s.tirar(&Pedido::novo("mwb", "T", "")));
}

fn baixado(p: &Pedido) -> Baixado {
    Baixado { pedido: p.clone(), arquivo: arquivo(), pasta: p.chave(), baixado_em: 1 }
}

#[test]
fn o_que_conferir() {
    use leitor_baixar::{pedidos_da_sincronizacao, Escolha, Selecao};
    let mut s = Selecao::default();
    s.escolher(Escolha { pedido: Pedido::novo("w", "T", ""), novas_edicoes: true });
    s.escolher(Escolha { pedido: Pedido::novo("lff", "T", ""), novas_edicoes: false });
    // Edição fixa antiga: é item comum, conferida sempre.
    s.escolher(Escolha { pedido: Pedido::novo("w", "T", "202301"), novas_edicoes: false });
    // Edição fixa que também cai na janela: uma vez só.
    s.escolher(Escolha { pedido: Pedido::novo("w", "T", "202611"), novas_edicoes: false });
    // Periódico sem "edições novas" e sem edição: pedido comum.
    s.escolher(Escolha { pedido: Pedido::novo("mwb", "E", ""), novas_edicoes: false });
    let mut r = Registro::default();
    for p in [Pedido::novo("w", "T", "202605"), Pedido::novo("w", "T", "202610"), Pedido::novo("lff", "T", "")] {
        r.baixados.insert(p.chave(), baixado(&p));
    }
    let chaves: Vec<String> = pedidos_da_sincronizacao(&s, &r, (2026, 11)).iter().map(|p| p.chave()).collect();
    assert_eq!(
        chaves,
        ["lff_T", "mwb_E", "w_T_202609", "w_T_202610", "w_T_202611", "w_T_202612", "w_T_202701", "w_T_202702", "w_T_202301"]
    );
    // A edição baixada de maio (fora da janela) não é mais perguntada.
    assert!(!chaves.contains(&"w_T_202605".to_string()));
}

#[test]
fn regra_das_24_horas() {
    use leitor_baixar::{precisa_verificar, INTERVALO};
    let agora = 1_800_000_000;
    assert!(precisa_verificar(0, agora), "nunca verificou");
    assert!(!precisa_verificar(agora, agora));
    assert!(!precisa_verificar(agora - INTERVALO + 1, agora), "23h59 ainda não");
    assert!(precisa_verificar(agora - INTERVALO, agora), "24 h em ponto");
    assert!(precisa_verificar(agora + 3600, agora), "relógio voltou: verifica");
}

#[test]
fn registro_antigo_sem_ultima_verificacao() {
    let r: Registro = serde_json::from_str(r#"{"baixados":{}}"#).unwrap();
    assert_eq!(r.ultima_verificacao, 0);
    let s: leitor_baixar::Selecao = serde_json::from_str(r#"{"escolhas":[]}"#).unwrap();
    assert!(!s.atualizar_sozinho);
}

#[test]
fn texto_do_aviso() {
    use leitor_baixar::{resumo, Atualizacao};
    let n = |atual: bool| Atualizacao { pedido: Pedido::novo("w", "T", "202611"), atual: atual.then(arquivo), novo: arquivo() };
    assert_eq!(resumo(&[]), None);
    assert_eq!(resumo(&[n(true)]).as_deref(), Some("1 publicação tem versão nova"));
    assert_eq!(resumo(&[n(true), n(true)]).as_deref(), Some("2 publicações têm versão nova"));
    assert_eq!(resumo(&[n(false)]).as_deref(), Some("1 publicação nova para baixar"));
    assert_eq!(
        resumo(&[n(true), n(false), n(false)]).as_deref(),
        Some("1 publicação tem versão nova e 2 publicações novas para baixar")
    );
}

