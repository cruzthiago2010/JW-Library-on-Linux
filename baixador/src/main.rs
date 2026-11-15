//! O baixador: sem argumentos abre a janela (feature "janela"); com
//! argumentos é a linha de comando.
//!
//!   jw-baixador consultar <sigla> <idioma> [edição]
//!   jw-baixador baixar    <sigla> <idioma> [edição]
//!   jw-baixador lista
//!   jw-baixador verificar [--instalar]
//!   jw-baixador catalogo | escolher | tirar | selecao | sincronizar
use std::io::Write;
use std::process::ExitCode;

use baixador::catalogo::{self, CATALOGO, IDIOMAS};
use baixador::{Cliente, Desfecho, Escolha, Pastas, Pedido, Registro, Selecao, Situacao};

const AJUDA: &str = "\
Baixador de .jwpub para o JW Library on Linux

  jw-baixador                                       abre a janela
  jw-baixador consultar <sigla> <idioma> [edição]   mostra o que o jw.org oferece
  jw-baixador baixar    <sigla> <idioma> [edição]   baixa e põe na Biblioteca
  jw-baixador lista                                 o que já foi baixado
  jw-baixador verificar [--instalar]                procura versões novas

  jw-baixador catalogo                              publicações e idiomas conhecidos
  jw-baixador escolher  <sigla> <idioma> [edição] [--novas]
                                                    põe na minha seleção (--novas: cada
                                                    edição nova de w/mwb)
  jw-baixador tirar     <sigla> <idioma> [edição]   tira da seleção
  jw-baixador selecao                               mostra a seleção
  jw-baixador sincronizar                           baixa o que falta e o que mudou
  jw-baixador conferir-catalogo <idioma>            quais siglas do catálogo existem

Exemplos: baixar nwtsty T · baixar w T 202609 · baixar mwb T 202611
Idioma pelo código do jw.org: T português, E inglês, S espanhol.";

fn pedido(args: &[String]) -> Result<Pedido, String> {
    match args {
        [sigla, lingua] => Ok(Pedido::novo(sigla, lingua, "")),
        [sigla, lingua, edicao] => Ok(Pedido::novo(sigla, lingua, edicao)),
        _ => Err("faltou <sigla> <idioma> [edição]".into()),
    }
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}

fn barra() -> impl FnMut(u64, u64) {
    use std::io::IsTerminal;
    // Fora de um terminal (log, pipe), de 10 em 10%.
    let passo = if std::io::stderr().is_terminal() { 1 } else { 10 };
    let mut ultimo = u64::MAX;
    move |feito, total| {
        let pct = (feito * 100).checked_div(total).unwrap_or(0) / passo * passo;
        if pct != ultimo {
            ultimo = pct;
            eprint!("\r  {pct:3}%  {} de {}", mb(feito), mb(total));
            let _ = std::io::stderr().flush();
        }
    }
}

fn rodar(args: Vec<String>) -> Result<(), String> {
    let Some(comando) = args.first() else {
        #[cfg(feature = "janela")]
        return baixador::janela::abrir();
        #[cfg(not(feature = "janela"))]
        {
            println!("{AJUDA}");
            return Ok(());
        }
    };
    let pastas = Pastas::padrao()?;
    let mut cliente = Cliente::novo();
    match comando.as_str() {
        "consultar" => {
            let p = pedido(&args[1..])?;
            match cliente.consultar(&p)? {
                Some(a) => {
                    println!("{}\n  {}\n  {} · {}\n  MD5 {}\n  modificado em {}", a.publicacao, a.titulo, a.nome().unwrap_or_default(), mb(a.tamanho), a.md5, a.modificado);
                }
                None => println!("{}: não há .jwpub nesse idioma", p.chave()),
            }
        }
        "baixar" => {
            let p = pedido(&args[1..])?;
            let mut registro = Registro::abrir(&pastas.registro())?;
            let r = baixador::baixar(&mut cliente, &pastas, &mut registro, &p, &mut barra())?;
            if r.situacao != Situacao::JaEstava {
                eprintln!();
            }
            let o_que = match r.situacao {
                Situacao::Novo => "baixado e instalado",
                Situacao::Atualizado => "atualizado",
                Situacao::JaEstava => "já estava em dia",
            };
            println!("{} ({}): {o_que} em {}", r.arquivo.publicacao, r.arquivo.titulo, pastas.biblioteca.join(&r.pasta).display());
        }
        "lista" => {
            let registro = Registro::abrir(&pastas.registro())?;
            if registro.baixados.is_empty() {
                println!("Nada baixado ainda.");
            }
            for (chave, b) in &registro.baixados {
                println!("{chave:24} {} · {} · {}", b.arquivo.publicacao, b.arquivo.titulo, mb(b.arquivo.tamanho));
            }
        }
        "verificar" => {
            let instalar = args.iter().any(|a| a == "--instalar");
            let mut registro = Registro::abrir(&pastas.registro())?;
            let (novas, erros) = baixador::verificar(&mut cliente, &registro);
            for e in &erros {
                eprintln!("erro: {e}");
            }
            if novas.is_empty() {
                let n = registro.baixados.len();
                println!("Tudo em dia ({n} {}).", if n == 1 { "publicação" } else { "publicações" });
            }
            for n in novas {
                println!("Versão nova: {} ({}) de {}", n.novo.publicacao, n.pedido.chave(), n.novo.modificado);
                if instalar {
                    baixador::baixar(&mut cliente, &pastas, &mut registro, &n.pedido, &mut barra())?;
                    eprintln!();
                }
            }
        }
        "catalogo" => {
            let mut categoria = "";
            for i in CATALOGO {
                if i.categoria != categoria {
                    categoria = i.categoria;
                    println!("\n{}", leitor_nucleo::biblioteca::nome_da_categoria(categoria));
                }
                let extra = match i.periodico {
                    Some(catalogo::Periodicidade::Mensal) => "  (mensal; edição AAAAMM)",
                    Some(catalogo::Periodicidade::Bimestral) => "  (meses ímpares; edição AAAAMM)",
                    None => "",
                };
                println!("  {:8} {}{extra}", i.sigla, i.nome);
            }
            println!("\nIdiomas:");
            for (codigo, nome) in IDIOMAS {
                println!("  {codigo:8} {nome}");
            }
        }
        "escolher" => {
            let novas = args.iter().any(|a| a == "--novas");
            let resto: Vec<String> = args[1..].iter().filter(|a| *a != "--novas").cloned().collect();
            let p = pedido(&resto)?;
            if novas && !p.edicao.is_empty() {
                return Err("--novas é para o periódico sem edição (ex.: escolher w T --novas)".into());
            }
            if novas && catalogo::achar(&p.sigla).and_then(|i| i.periodico).is_none() {
                return Err(format!("{} não é periódico do catálogo (w ou mwb)", p.sigla));
            }
            let mut selecao = Selecao::abrir(&pastas.selecao())?;
            let chave = p.chave();
            let mudou = selecao.escolher(Escolha { pedido: p, novas_edicoes: novas });
            selecao.salvar(&pastas.selecao())?;
            println!("{chave}: {}", if mudou { "na seleção" } else { "já estava na seleção" });
        }
        "tirar" => {
            let p = pedido(&args[1..])?;
            let mut selecao = Selecao::abrir(&pastas.selecao())?;
            let tirou = selecao.tirar(&p);
            selecao.salvar(&pastas.selecao())?;
            println!("{}: {}", p.chave(), if tirou { "saiu da seleção (o que já foi baixado fica na Biblioteca)" } else { "não estava na seleção" });
        }
        "selecao" => {
            let selecao = Selecao::abrir(&pastas.selecao())?;
            if selecao.escolhas.is_empty() {
                println!("A seleção está vazia. Ex.: jw-baixador escolher nwtsty T");
            }
            for e in &selecao.escolhas {
                let nome = catalogo::achar(&e.pedido.sigla).map(|i| i.nome).unwrap_or("");
                let novas = if e.novas_edicoes { "  + edições novas" } else { "" };
                println!("{:24} {nome}{novas}", e.pedido.chave());
            }
        }
        "sincronizar" => {
            let selecao = Selecao::abrir(&pastas.selecao())?;
            let mut registro = Registro::abrir(&pastas.registro())?;
            let mut atual = String::new();
            let mut desenho = barra();
            let desfechos = baixador::sincronizar(&mut cliente, &pastas, &mut registro, &selecao, &mut |p, f, t| {
                if atual != p.chave() {
                    if !atual.is_empty() {
                        eprintln!(); // fecha a barra de progresso do anterior
                    }
                    atual = p.chave();
                    eprintln!("{atual}");
                }
                desenho(f, t)
            });
            if !atual.is_empty() {
                eprintln!();
            }
            let (mut feitos, mut erros) = (0, 0);
            for d in desfechos {
                match d {
                    Desfecho::Feito(r) => {
                        let o_que = match r.situacao {
                            Situacao::Novo => "novo",
                            Situacao::Atualizado => "atualizado",
                            Situacao::JaEstava => "em dia",
                        };
                        if r.situacao != Situacao::JaEstava {
                            feitos += 1;
                        }
                        // O título às vezes repete o nome da publicação (ex.: lff).
                        let titulo = if r.arquivo.titulo == r.arquivo.publicacao { String::new() } else { format!(" ({})", r.arquivo.titulo) };
                        println!("{:24} {o_que}  {}{titulo}", r.pasta, r.arquivo.publicacao);
                    }
                    Desfecho::NaoSaiu(_) => {}
                    Desfecho::Erro(p, e) => {
                        erros += 1;
                        eprintln!("erro: {}: {e}", p.chave());
                    }
                }
            }
            println!("Pronto: {feitos} baixado(s), {erros} erro(s).");
        }
        "conferir-catalogo" => {
            let lingua = args.get(1).map(String::as_str).unwrap_or("T");
            let (ano, mes) = catalogo::hoje();
            for i in CATALOGO {
                let edicao = match i.periodico {
                    Some(p) => catalogo::edicoes_perto(p, ano, mes).first().cloned().unwrap_or_default(),
                    None => String::new(),
                };
                let p = Pedido::novo(i.sigla, lingua, &edicao);
                match cliente.consultar(&p) {
                    Ok(Some(a)) => println!("ok     {:20} {} · {}", p.chave(), a.publicacao, mb(a.tamanho)),
                    Ok(None) => println!("FALTA  {:20} {}", p.chave(), i.nome),
                    Err(e) => println!("ERRO   {:20} {e}", p.chave()),
                }
            }
        }
        "-h" | "--help" | "ajuda" => println!("{AJUDA}"),
        outro => return Err(format!("comando desconhecido: {outro}\n\n{AJUDA}")),
    }
    Ok(())
}

fn main() -> ExitCode {
    match rodar(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
}
