# JW Library on Linux 0.8.0

- **Texto diário no Início**, como no JW Library: data, setas para outros dias, o texto e o começo do comentário; um clique abre o dia inteiro. Sem o "Examine as Escrituras Diariamente", o cartão oferece baixar. / **Daily text** on Home.
- Pronto para o **Flathub**: compila sem internet e o identificador do Flatpak passa a ser `io.github.cruzthiago2010.JW-Library-on-Linux` (quem instalou o Flatpak antigo deve instalar o novo; publicações e grifos continuam na mesma pasta). / Ready for Flathub; new Flatpak app ID.

## Instalar · Install

- Flatpak: `flatpak install --user JW_Library_on_Linux-0.8.0-x86_64.flatpak` (ARM: `-aarch64.flatpak`)
- AppImage: `chmod +x JW_Library_on_Linux-0.8.0-x86_64.AppImage` e abra / and run it (ARM: `-aarch64.AppImage`)

# JW Library on Linux 0.7.0

- **Notícias no Início**, como no JW Library: a faixa "NOVO VÍDEO | …" com o texto completo ao abrir. / **News banner** on Home.
- **Vídeos e áudios sem internet e com busca**: o catálogo de mídia do jw.org fica no computador (as abas abrem na hora) e dá para buscar por nome. / Offline media catalog with search.
- **Todos os idiomas do jw.org** (mais de mil, inclusive línguas de sinais) para publicações, vídeos e áudios, com busca no seletor. / **All jw.org languages**, including sign languages.

## Instalar · Install

- Flatpak: `flatpak install --user JW_Library_on_Linux-0.7.0-x86_64.flatpak` (ARM: `-aarch64.flatpak`)
- AppImage: `chmod +x JW_Library_on_Linux-0.7.0-x86_64.AppImage` e abra / and run it (ARM: `-aarch64.AppImage`)

# JW Library on Linux 0.6.0

- **Biblioteca igual à do JW Library**: todas as categorias do catálogo do jw.org (Livros, Brochuras, Folhetos, Séries, A Sentinela, Despertai!, Apostilas, Ministério do Reino, Programas…), com capas. As publicações que você não tem aparecem com a nuvem e o tamanho: um clique baixa. Seletor de idioma das publicações. / **Library like JW Library**: the whole jw.org catalog, one-click download.
- **Abas VÍDEO e ÁUDIO**: as categorias do jw.org (De Nosso Estúdio, Crianças, Família, Músicas, Cante de Coração para Jeová…), assistir e ouvir dentro do app, baixar para ver sem internet e **Bíblia em áudio**. / **Video and Audio tabs**, offline downloads and the audio Bible.
- "Atualizações automáticas" (na barra da Biblioteca): revistas e apostilas cujas edições novas são baixadas sozinhas; no "..." de uma revista, "Baixar as edições novas sozinho". / Automatic updates for magazines and workbooks.

## Instalar · Install

- Flatpak: `flatpak install --user JW_Library_on_Linux-0.6.0-x86_64.flatpak` (ARM: `-aarch64.flatpak`)
- AppImage: `chmod +x JW_Library_on_Linux-0.6.0-x86_64.AppImage` e abra / and run it (ARM: `-aarch64.AppImage`)

# JW Library on Linux 0.5.0

- **Aba BAIXAR na Biblioteca** (o antigo Baixador, agora embutido): catálogo, Minha seleção, "Baixar agora", procurar atualizações uma vez por dia e aviso de edição nova ou arquivo atualizado. Quem usava o Baixador separado tem a seleção trazida sozinha. / **DOWNLOAD tab** in the Library (the former Downloader, now built in).
- Excluir publicação pelo menu "..." do cartão (Abrir · Excluir), como no JW Library. / Delete a publication from the card's "..." menu.
- Janela única: abrir o app de novo traz a janela aberta para a frente. / Single window.

## Instalar · Install

- Flatpak: `flatpak install --user JW_Library_on_Linux-0.5.0-x86_64.flatpak` (ARM: `-aarch64.flatpak`)
- AppImage: `chmod +x JW_Library_on_Linux-0.5.0-x86_64.AppImage` e abra / and run it (ARM: `-aarch64.AppImage`)

# JW Library on Linux 0.4.0

- **Flatpak** (x86_64 e ARM64): instala em qualquer distribuição. O .deb deixa de existir; ficam **Flatpak** e **AppImage**. / Flatpak packages; the .deb is gone, Flatpak and AppImage remain.
- Flatpak e AppImage usam a **mesma pasta** de publicações, grifos e notas (`~/.local/share/io.github.leitorjw`). / Both use the same data folder.
- **Excluir publicação** como no JW Library: menu "..." do cartão (Abrir · Excluir). / Delete a publication from the card's "..." menu.

## Instalar · Install

- Flatpak: `flatpak install --user JW_Library_on_Linux-0.4.0-x86_64.flatpak` (ARM: `-aarch64.flatpak`)
- AppImage: `chmod +x JW_Library_on_Linux-0.4.0-x86_64.AppImage` e abra / and run it (ARM: `-aarch64.AppImage`)

# JW Library on Linux 0.3.0

- **Busca nas publicações**: a lupa da Biblioteca e do Início procura em todos os artigos e capítulos importados (a aba Bíblia continua buscando versículos). / **Search publications**: search every imported article and chapter.
- **Remover publicação**: botão de lixeira na tela da publicação, com confirmação. Grifos e notas ficam guardados. / **Remove a publication** from the library; highlights and notes are kept.
- **ARM64**: instaladores também para computadores ARM (Raspberry Pi 4/5 com sistema de 64 bits e outros). / Installers for ARM64 too.
- Mensagens de erro em inglês quando a interface está em inglês. / Error messages follow the interface language.
- Integração com o baixador de publicações (projeto irmão): o botão "Baixar publicações" aparece quando ele está instalado, e a Biblioteca se atualiza ao voltar para a janela.

## Instalar · Install

- x86_64: `leitor-jw_0.3.0_amd64.deb` ou `JW_Library_on_Linux-0.3.0-x86_64.AppImage`
- ARM64: `leitor-jw_0.3.0_arm64.deb` ou `JW_Library_on_Linux-0.3.0-aarch64.AppImage`

# JW Library on Linux 0.2.0

- **Interface em inglês e português.** Na primeira abertura o app pergunta o idioma; dá para trocar em Configurações. Os nomes dos livros da Bíblia e das categorias da Biblioteca também mudam.
- **English and Portuguese interface.** The app asks for the language on first launch; you can change it in Settings. Bible book names and Library categories follow the chosen language.

## Instalar · Install

- Debian, Ubuntu, Mint: `sudo apt install ./leitor-jw_0.2.0_amd64.deb`
- Outras distribuições · Other distributions: `JW_Library_on_Linux-0.2.0-x86_64.AppImage`

# JW Library on Linux 0.1.2

- Logotipo e ícone novos.

# JW Library on Linux 0.1.1

- O app passa a se chamar **JW Library on Linux** (antes "Leitor JW"). Grifos, notas e publicações continuam no mesmo lugar.

# 0.1.0 — primeira versão

Leitor de publicações `.jwpub` para Linux, no estilo do JW Library do Windows. Funciona sem internet.

- Biblioteca por categorias, anos e edições; importar `.jwpub` pelo botão, arrastando ou pelo aviso da pasta Downloads.
- Leitor com painel de estudo (notas de rodapé, textos citados), tamanho do texto, tela inteira e tema escuro.
- Bíblia com notas de estudo, referências marginais e busca.
- Reuniões: apostila e Sentinela da semana.
- Grifos em 6 cores, notas com etiquetas, 10 marcadores e campos de resposta.
- Backup `.jwlibrary` no mesmo formato do JW Library (leva e traz grifos e notas do celular).

## Instalar

- Debian, Ubuntu, Mint: baixe `leitor-jw_0.1.2_amd64.deb` e rode `sudo apt install ./leitor-jw_0.1.2_amd64.deb`.
- Outras distribuições: baixe `JW_Library_on_Linux-0.1.2-x86_64.AppImage`, dê permissão de execução e abra.

Só para computadores de 64 bits (x86_64). Nenhuma publicação vem junto: importe os arquivos `.jwpub` que você já tem.
