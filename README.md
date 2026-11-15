# JW Library for Linux (Leitor JW)

Leitor de publicações `.jwpub` no estilo do JW Library do Windows, para quem usa Linux.
Funciona sem internet: você importa os arquivos `.jwpub` que já tem (baixados no jw.org ou copiados
de outro aparelho).

Projeto da comunidade, **sem ligação oficial** com a organização. O app não traz publicações nem o
CSS do JW Library: o visual das publicações vem de um CSS próprio (`ui/estilo/`).

## O que faz

- **Biblioteca** por categoria (Bíblias, Livros, A Sentinela, Apostilas…), anos e edições, aba Baixados
  com tamanho e ordenação. Importa `.jwpub` pelo botão, arrastando para a janela ou pelo aviso de
  arquivo novo na pasta Downloads.
- **Leitor** com painel de estudo (notas de rodapé, trechos citados, textos bíblicos), setas de artigo,
  tamanho do texto, tela inteira, tema claro/escuro.
- **Bíblia**: livros, capítulos, notas de estudo, referências marginais, menu do versículo e busca.
- **Reuniões**: a apostila e o estudo de A Sentinela da semana, com setas de semana.
- **Estudo Pessoal**: grifos em 6 cores, notas com etiquetas, marcadores (10 fitas), campos de
  resposta, tela "Notas e etiquetas".
- **Backup** `.jwlibrary` no mesmo formato do JW Library: dá para levar grifos e notas para o
  celular ou trazer de lá.

## Instalar

- Debian, Ubuntu, Mint: `sudo apt install ./leitor-jw_0.1.0_amd64.deb`
- Outras distribuições: dê permissão de execução ao `Leitor_JW-0.1.0-x86_64.AppImage` e abra.

Os dados ficam em `~/.local/share/io.github.leitorjw/` (`biblioteca/` com as publicações e
`userData.db` com grifos e notas, no formato do JW Library).

## Pastas do código

- `nucleo/` — sem Tauri, testável: `.jwpub` (`jwpub.rs`), Biblioteca
  (`biblioteca.rs`), leitor (`leitor.rs`), painel (`estudo.rs`), Reuniões (`reunioes.rs`), Bíblia e
  busca (`biblia.rs`, `busca.rs`), estudo pessoal (`pessoal.rs` + `userdata.sql`, o esquema v16 do
  JW Library), tokens dos grifos (`tokens.rs`) e backup (`backup.rs`).
- `app/` — a casca Tauri 2 (comandos e o protocolo `conteudo://` das imagens).
- `ui/` — a janela (HTML/CSS/JS puro). `marcas.js` pinta os grifos com a API de destaques do CSS.
- `ferramentas/` — `gerar_css.py` (gera `ui/estilo/utilidades.css`) e `print.sh` (prints numa tela virtual).
- `docs/` — inventário das telas do JW Library usado como referência de desenho.

## Compilar

```bash
cargo test -p leitor-nucleo          # LEITOR_BIBLIOTECA=data/biblioteca LEITOR_BACKUP=<arquivo.jwlibrary> para testar com dados reais
cargo run -p leitor-jw
cd app && cargo tauri build --bundles deb,appimage   # pacotes (precisa de ~3 GB livres)
```

Precisa de `libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `pkg-config` e `libssl-dev`.

## Aviso

Este é um projeto independente, feito por voluntários, **sem nenhuma ligação** com a Watch Tower
Bible and Tract Society nem com o JW Library. Nenhuma publicação acompanha o app: ele só abre
arquivos `.jwpub` que você mesmo já tem. Os nomes citados pertencem aos seus donos.

## Licença

GPL-3.0-or-later (veja `LICENSE`).
