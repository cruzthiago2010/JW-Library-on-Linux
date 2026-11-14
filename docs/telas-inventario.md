# Inventário de telas — JW Library × Reuniões (Etapa 1)

Capturas feitas em 13/11/2026 no Galaxy Tab A7 Lite (800×1340, retrato, tema escuro), por adb.
PNGs em `data/telas/jwlibrary/` e `data/telas/reunioes/` (fora do git: o conteúdo das publicações
é de uso pessoal). Aqui ficam só as descrições de layout, sem texto das publicações.

Status no Reuniões: **tem igual** · **tem parecido** · **falta**.

## 1. Estrutura geral do JW Library

- **Barra de cima** (≈ 0–115 px): seta "voltar/subir" à esquerda, título em 2 linhas (título + subtítulo
  com publicação ou idioma), ícones à direita (variam por tela: áudio, buscar, marcadores, baixar mídias,
  idiomas, histórico, atualizar, configurações) e menu ⋮ "Mais opções".
- **Barra de baixo fixa** (≈ 1200–1276 px) com 5 abas: Início, Bíblia, Biblioteca, Reuniões, Estudo Pessoal.
  Cada aba guarda a própria pilha de navegação (voltar à aba devolve onde se parou; o ← sobe um nível).
- **Painel de estudo à direita** no leitor (≈ 280 px de 800): referências, notas de rodapé, conteúdo do
  livro, notas pessoais. Botão redondo flutuante "→|" esconde/mostra o painel.
- Pop-ups de lista (menus ⋮) ancorados no canto superior direito; diálogos centrais com botão FECHAR.

## 2. Telas do JW Library

| # | Tela | Como chegar | Layout / elementos | Reuniões |
|---|---|---|---|---|
| 01–02 | **Início** | aba Início | Faixa de "novo vídeo" no topo; bloco do texto do dia (ou convite para baixar); Favoritos (grade vazia); "Kit de Ensino" em carrossel horizontal de cartões (miniatura, duração, ⋮, nuvem de download); Novidades; On-line (atalhos). Ícones: histórico, atualizar, configurações | **falta** (o Reuniões abre direto na semana) |
| 03 | **Bíblia – livros** | aba Bíblia | Abas roláveis (Introdução, Livros, Índice, Apêndices A/B/C). Grade de livros 6 colunas com abreviação, dividida em Hebraico‑Aramaicas / Gregas Cristãs; cor por grupo de livros | **tem parecido** (grade de livros com cores, sem as abas de introdução/apêndices) |
| 04 | Bíblia – menu ⋮ | ⋮ na Bíblia | Histórico, Compartilhar link | **falta** |
| 05 | **Bíblia – capítulos** | toque num livro | Grade de capítulos 5 colunas na metade esquerda + "Introdução a…"; à direita "Conteúdo do livro" (sumário por capítulo com faixas de versículos) | **tem parecido** (grade 6 colunas em tela cheia, sem o sumário ao lado) |
| 06 | **Capítulo aberto** | toque no capítulo | Texto à esquerda (número grande do capítulo, versículos numerados, letras de referência e * de notas); painel de estudo à direita (Guia de Pesquisa, conteúdo do livro, referências e notas por versículo). Setinhas ▲▼ no topo para capítulo anterior/seguinte | **tem parecido** (texto em tela cheia, sem painel lateral nem setas de capítulo) |
| 07 | Toque numa letra de referência | toque no "a", "b"… | Painel da direita vira "Referências marginais" com o versículo citado inteiro e seta > para abrir | **tem parecido** (abre faixa na parte de baixo com a referência) |
| 08 | **Toque no número do versículo** | toque no número | Versículo realçado, resto esmaecido; barra flutuante de ícones: grifar (Estudo), nota, comparar traduções, marcador, compartilhar, copiar, áudio | **falta** |
| 09 | **Seleção de texto / menu de grifo** | toque longo numa palavra | Alças de seleção azuis; barra flutuante com 6 cores de grifo (amarelo, verde, azul, roxo, rosa, laranja), adicionar nota, copiar, buscar, "Assuntos" | **falta** (no Reuniões o toque longo não abre menu nenhum) |
| 10 | Capítulo sem painel | botão "→\|" | Texto ocupa a largura toda; botão vira "Mostrar painel" | **tem igual** (o Reuniões é sempre largura toda) |
| 11 | Leitor – menu ⋮ | ⋮ no leitor | Adicionar uma nota, Histórico, Compartilhar link, Configurações de texto, Tela inteira | **falta** (só o "Aa") |
| 12 | **Configurações de texto** | ⋮ › Configurações de texto | Diálogo com A pequeno — controle deslizante — A grande, FECHAR | **tem igual** ("Aa": deslizante + Padrão/FECHAR) |
| 13–14 | **Busca na publicação** | ícone lupa no leitor | Campo de busca na barra de cima + teclado; resultados com abas (Versículos mais usados / Todos os versículos / Artigos), contador de ocorrências, cartões com referência e trecho com a palavra em negrito, opção "achar a expressão exata" | **tem parecido** (Bíblia do Reuniões tem lupa "Buscar na Bíblia"; sem abas nem contador) |
| 15 | **Marcadores** | ícone marcador | Diálogo "Marcadores – publicação" com 10 faixas coloridas (vazias = "não usado"), FECHAR | **falta** |
| 16 | **Biblioteca – publicações** | aba Biblioteca | Abas: Publicações, Vídeo, Áudio, Baixados, Atualizações pendentes. Lista de categorias com ícone (Livros, Brochuras, Folhetos, Séries, A Sentinela, Despertai!, Apostilas, Ministério do Reino) | **tem parecido** (Arquivo: lista por semana com selos; não por categoria) |
| 17 | Biblioteca – baixados | aba Baixados | Lista agrupada por tipo; cartão com capa, título, ano, idioma, tamanho e ⋮ | **tem parecido** (Arquivo mostra o que está pronto por semana) |
| 18 | Biblioteca – vídeo | aba Vídeo | Botão "Assistir agora" e categorias com imagem grande | **falta** |
| 19 | **Lista de livros** | Biblioteca › Livros | Cartões largos: capa à esquerda, título, ano, ⋮ e nuvem/tamanho para baixar. Ícones: idioma, ordenar, histórico | **falta** |
| 21–22 | **Publicação – sumário** | toque numa publicação | Capa grande no topo, título, abas por seção; lista de capítulos com miniatura e ⋮ | **falta** (Reuniões não navega publicações inteiras) |
| 23 | **Publicação – capítulo** | toque num capítulo | Leitor com o CSS da publicação (faixa colorida, cabeçalho do capítulo, títulos, tabelas, imagens) + painel lateral com notas da publicação | **tem parecido** (camada do estudo de livro usa HTML/CSS do .jwpub, sem painel) |
| 24–25b | **Reuniões – semana** | aba Reuniões | Seletor de semana "‹ data · Esta semana ›"; blocos Vida e Ministério, Estudo de A Sentinela (miniatura + título + ⋮); "Outras publicações usadas" (apostila, revista, cânticos, brochuras com nuvem de download). ⋮ = reproduzir áudio, baixar áudio, compartilhar link | **tem parecido** (Reuniões: abas Meio/Fim de semana com o conteúdo já aberto, setas de semana) |
| 26 | **Apostila (meio de semana)** | toque em Vida e Ministério | Leitor da apostila com faixas coloridas das 3 seções, imagem, partes numeradas com tempo; links azuis de textos e publicações; painel lateral "Referências" | **tem igual/parecido** (modo JW: mesmo HTML/CSS, com caixas "Sua resposta"; sem painel) |
| 27 | **Sentinela (fim de semana)** | toque no Estudo de A Sentinela | Faixa laranja com a data, cântico, título, texto-tema, quadro Objetivo, perguntas numeradas e caixas de resposta; painel lateral | **tem igual** (modo JW do fim de semana, com respostas preenchidas) |
| 29 | Toque num texto bíblico no artigo | link azul de versículo | Texto realçado; painel da direita mostra o versículo com "Comparar versões" e "Personalizar" | **tem parecido** (Reuniões abre folha de baixo com o texto e botões Voltar / Capítulo inteiro / Abrir no JW Library) |
| 30 | **Estudo Pessoal** | aba Estudo Pessoal | Seções "Notas e etiquetas" (cartões de nota coloridos, + etiqueta, + nota) e "Playlists"; ícones Backup e restauração, Histórico | **falta** (o Reuniões tem só "Enviar ao JW", que lê o backup) |
| 31, 34 | **Notas e etiquetas** | › Notas e etiquetas | Busca; aviso de etiquetas; grade de 2 colunas de cartões (data, trecho grifado em negrito, texto da nota, ⋮, rodapé com capa e origem); cor do cartão = cor do grifo | **falta** |
| 32 | Nota aberta no leitor | toque no rodapé de uma nota | Leitor na posição; a nota aparece no painel da direita como cartão colorido com ⋯ | **falta** |
| 33 | Nota na Bíblia | Bíblia › Romanos 14 | Quadradinho amarelo na margem do versículo com nota; nota em cartão colorido no painel, abaixo das referências | **falta** |
| 33b | Nota de publicação não baixada | toque numa nota de outra Bíblia | Diálogo "Baixar … (tamanho)" CANCELAR/BAIXAR | — (só registro) |
| 35–36 | **Busca geral** | Biblioteca › lupa | Campo "Assuntos, publicações e mídias", sugestões em cartões, rodapé "Buscar no JW.ORG" | **falta** |
| 37 | **Configurações** | Início › engrenagem | Lista com seções (Exibição: tema; Exibição e downloads: dados móveis, modo off-line, onde salvar; Playlists; Privacidade) | **tem parecido** (Ajustes: tema Escuro/Claro/Automático, dias das reuniões, aparelhos Android) |
| 38 | **Histórico** | ícone relógio | Diálogo com os últimos lugares lidos (título + publicação) | **falta** |

## 3. Telas do Reuniões (app atual, TWA)

| # | Tela | Observações de layout |
|---|---|---|
| 01–03 | Semana – meio de semana, **desenho antigo** | Barra: ‹ data › + "Semana atual", avisos (com número), Aa, PDF, "expandir/recolher tudo". Abas Meio/Fim de semana. Partes em cartões que expandem (seta ˅), caixa "Sem comentário ainda / Escrever". Barras de cima e de baixo somem ao rolar |
| 12–14 | Semana – **modo JW** | Mesmo conteúdo com HTML/CSS da apostila e da Sentinela, partes já abertas, caixas "Sua resposta" + "Comentários adicionais" + Editar. Aparece sozinho quando a semana tem o .jwpub (o botão "expandir/recolher" some) |
| 04 | Texto bíblico | Folha que sobe de baixo: referência, texto, botões Voltar / Capítulo inteiro / Abrir no JW Library |
| 05–08 | Camada do estudo de livro | Barra "Estudo de livro" + nome do livro, Aa, avisos; capítulo com CSS da publicação; "Medite no que aprendeu" com caixas de resposta. Aa = diálogo com deslizante, Padrão, FECHAR |
| 09 | Avisos | Lista dos últimos 30 dias (título, resumo, data/hora), ponto azul nos não lidos |
| 15–17c | Bíblia | Grade de livros colorida, grade de capítulos 6 colunas, capítulo em tela cheia; letra de referência abre faixa embaixo; toque longo não faz nada |
| 18 | Arquivo | Busca; cartões por semana com selos (Apostila ✓, Sentinela ✓, toolsjw n/6, preparada) e "Abrir ›" |
| 20 | Enviar ao JW | Passo a passo, semana inicial, o que levar, escolher arquivo .jwlibrary |
| 22 | Ajustes | Tema, dias das reuniões (Quarta / **Domingo** — o painel diz sábado, pendência antiga), app Android e aparelhos ligados |

Coisas notadas no Reuniões durante a navegação (para a versão nativa):
- O destaque da barra de baixo fica "preso" na aba anterior depois de voltar com o ←/voltar do Android.
- Tocar em "Semana" estando em Ajustes às vezes não troca a tela (só o destaque muda); o voltar do Android resolve.
- Aviso "Nova versão do app · Atualizar" apareceu durante o uso.

## 4. Resumo tem/falta

| Tela / recurso do JW Library | Reuniões |
|---|---|
| Barra de 5 abas fixa | tem parecido (5 abas, mas Semana/Bíblia/Arquivo/Enviar/Ajustes; some ao rolar) |
| Início (texto do dia, favoritos, kit) | falta |
| Bíblia: livros e capítulos | tem parecido |
| Bíblia: capítulo aberto | tem parecido |
| Painel de estudo lateral (referências, notas de rodapé, notas pessoais) | falta |
| Referência/letra → texto citado | tem parecido (folha de baixo) |
| Menu do versículo (toque no número) | falta |
| Seleção de texto + 6 cores de grifo + nota | falta |
| Grifos visíveis no texto | tem parecido (grifos da Sentinela vêm prontos; não se cria grifo no app) |
| Notas pessoais no texto (marca na margem + cartão) | tem parecido (caixas de comentário por pergunta, não notas livres) |
| Tamanho do texto | tem igual |
| Busca dentro da publicação/Bíblia | tem parecido |
| Busca geral | falta |
| Marcadores | falta |
| Histórico de leitura | falta |
| Biblioteca por categoria / lista / sumário de publicação | falta (Arquivo é por semana) |
| Reuniões: seletor de semana | tem igual |
| Reuniões: apostila e Sentinela com CSS do JW | tem igual (modo JW) |
| Estudo de livro (camada) | tem igual |
| Estudo Pessoal: notas, etiquetas, playlists | falta |
| Backup e restauração | tem parecido (Enviar ao JW) |
| Configurações | tem parecido |
| Vídeo / áudio / playlists | falta |
| Avisos | só no Reuniões |
| PDF da reunião | só no Reuniões |
