# Baixador de .jwpub (companheiro do JW Library on Linux)

Etapas 1 e 2: só a linha de comando (a janela vem na Etapa 3). Baixa um `.jwpub` pela API pública de download do jw.org
(`GETPUBMEDIALINKS`), confere o MD5 e instala direto na Biblioteca do JW Library on Linux
(`~/.local/share/io.github.leitorjw/biblioteca/`). O registro do que foi baixado fica em
`~/.local/share/io.github.leitorjw.baixador/registro.json`.

```bash
cd baixador
CARGO_TARGET_DIR=../target cargo test
CARGO_TARGET_DIR=../target cargo run -- baixar nwtsty T
CARGO_TARGET_DIR=../target cargo run -- baixar w T 202609
CARGO_TARGET_DIR=../target cargo run -- verificar

# Etapa 2: catálogo e seleção
CARGO_TARGET_DIR=../target cargo run -- catalogo
CARGO_TARGET_DIR=../target cargo run -- escolher nwtsty T
CARGO_TARGET_DIR=../target cargo run -- escolher mwb T --novas   # cada apostila nova
CARGO_TARGET_DIR=../target cargo run -- sincronizar
```

Uma pergunta por segundo, no máximo, e um User-Agent que diz o que é o programa.
Os termos de uso do jw.org restringem programas que baixam conteúdo do site: uso pessoal.
