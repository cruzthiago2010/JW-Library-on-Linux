# Baixador de .jwpub para o JW Library on Linux — plano por etapas

Criado em 15/11/2026. Segue a regra do projeto JW desktop: as etapas andam sem perguntar;
publicar no GitHub e qualquer coisa irreversível só com o "ok" do Thiago.

## O que é

Um segundo app (Rust + Tauri 2, mesmo visual roxo do JW Library on Linux) em que a pessoa:

1. escolhe as publicações e o idioma (Bíblia de estudo, livros, brochuras, A Sentinela, apostilas…);
2. baixa os `.jwpub` e eles já aparecem na Biblioteca do JW Library on Linux;
3. recebe um aviso quando sai uma versão nova de um arquivo baixado, ou uma edição nova de um
   periódico que ela marcou (Sentinela, apostila).

## Como os dois apps conversam

O JW Library on Linux guarda as publicações em `~/.local/share/io.github.leitorjw/biblioteca/`
e instala com `leitor_nucleo::jwpub::instalar` (a mesma função do botão Importar).
O baixador usa **o mesmo núcleo** (`nucleo/`, dependência por caminho) e instala direto nessa
pasta. Resultado: nada de passar pela pasta Downloads, e a Biblioteca mostra a publicação na próxima
vez que é aberta. O registro do baixador (o que foi escolhido, o checksum e a data de cada arquivo)
fica separado, em `~/.local/share/io.github.leitorjw.baixador/`.

## De onde vêm os arquivos (e os termos do jw.org)

Só a API pública que os próprios botões de download do jw.org usam:
`https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS?pub=…&issue=…&langwritten=…&fileformat=JWPUB`.
Ela devolve a URL, o tamanho, o MD5 e a data de modificação de cada arquivo, o que serve para
conferir o download e para perceber atualização. Nada de raspar páginas do site.
Cuidados: no máximo uma pergunta por segundo, User-Agent que diz o que é o app, cache das respostas,
verificação de atualização no máximo uma vez por dia.

**Atenção:** os termos de uso do jw.org proíbem distribuir programas feitos para baixar conteúdo
do site. Para uso pessoal o risco é baixo; para publicar para a comunidade, a decisão é sua e fica
para a Etapa 6 (nada vai para o GitHub sem o seu "ok").

## Onde fica o código

Subpasta nova `baixador/` dentro de "JW for linux", com `[workspace]` próprio, para não mexer no
`Cargo.toml` da raiz enquanto o outro thread trabalha na tradução e no logotipo. Usa a mesma pasta
de compilação (`target/`), então quase não gasta disco. Na Etapa 5 ela vira membro do workspace e
sai na mesma release.

## Etapas

**Etapa 1 — Núcleo do download (sem janela).** Cliente da API (ritmo de 1 por segundo, cache),
download para um `.part` com MD5 conferido e troca no fim, instalação na Biblioteca pelo núcleo do
leitor, registro em JSON. Linha de comando para testar: `baixador baixar nwtsty T`,
`baixador baixar w T 202609`. Testes do formato da resposta com um JSON salvo.

**Etapa 2 — Catálogo para escolher.** Lista embutida por categoria (as mesmas da Biblioteca), com
nome e sigla; periódicos com anos e edições (a apostila é bimestral, a Sentinela é mensal); idioma
(português, inglês, espanhol… pelo código do jw.org: T, E, S); campo para digitar uma sigla que não
está na lista. A "minha seleção" fica salva.

**Etapa 3 — A janela.** Telas Catálogo, Minha seleção e Baixados: capa (a API manda a imagem),
tamanho, barra de progresso, espaço livre no disco, apagar um baixado. Mesmo visual do leitor.

**Etapa 4 — Atualizações.** Compara MD5 e data com o registro; ao abrir o app e uma vez por dia.
"Sempre baixar edições novas" para Sentinela e apostila. Aviso do sistema (notificação do desktop).
Opcional: um timer `systemd --user` que roda `baixador verificar` sem abrir a janela.

**Etapa 5 — Junto com o JW Library on Linux.** Entra no workspace e no mesmo pacote; botão
"Baixar publicações" na Biblioteca do leitor que abre o baixador; a Biblioteca se atualiza sozinha
quando chega publicação nova. (Pequena mudança no app principal, combinada com o outro thread.)

**Etapa 6 — Pacote e publicação.** `.deb`/AppImage, README em português e inglês, aviso de projeto
não oficial. **Publicar no GitHub só com o seu "ok"**, depois de você decidir sobre os termos.

## Como vou testar

O contêiner onde trabalho não alcança o jw.org, então a compilação e os downloads de teste rodam no
seu PC por uma sessão de controle remoto, na pasta "JW for linux".
