# Leitor JW (para Linux)

Leitor de publicações `.jwpub` no estilo do JW Library do Windows, para quem usa Linux.
Funciona sem internet: você importa os arquivos `.jwpub` que já tem.

Projeto da comunidade, sem ligação oficial com a organização. Não traz publicações nem o CSS do
JW Library: o visual das publicações vem de um CSS próprio (`ui/estilo/`).

## Pastas

- `nucleo/` — sem Tauri, testável: abre e decifra o `.jwpub` (`jwpub.rs`, copiado do app Reuniões
  nativo), Biblioteca por categoria (`biblioteca.rs`) e o HTML do leitor (`leitor.rs`).
- `app/` — a casca Tauri 2: comandos `listar`, `importar`, `sumario`, `documento`, `achar_meps`
  e o protocolo `conteudo://` que serve as imagens.
- `ui/` — a janela (HTML/CSS/JS puro): trilho de 5 seções, barra de comandos, leitor.
- `ferramentas/gerar_css.py` — gera `ui/estilo/utilidades.css` (as classes `du-*` do HTML das
  publicações com valores nossos).
- `docs/` — plano por etapas e inventário das telas do JW Library.

## Compilar e abrir

```bash
export CARGO_TARGET_DIR="$HOME/Documents/Aplicativo Thiago/nativo/target" CARGO_INCREMENTAL=0   # no PC do Thiago: reaproveita a compilação e poupa disco
cargo test -p leitor-nucleo
cargo run -p leitor-jw
```

As publicações importadas ficam em `~/.local/share/io.github.leitorjw/biblioteca/`.
