// Idioma da interface: português (Brasil) ou inglês. O texto em português é a chave;
// t('…') devolve a tradução quando o idioma é inglês. {n}, {x}… são trocados pelos valores.
// O idioma das publicações não muda: é o do arquivo .jwpub.
const IDIOMAS = { pt: { nome: 'Português (Brasil)', local: 'pt-BR' }, en: { nome: 'English', local: 'en-US' } };

const Idioma = (() => {
  let atual = null;
  try { atual = JSON.parse(localStorage.getItem('idioma')); } catch (_) {}
  if (!IDIOMAS[atual]) atual = null;
  return {
    get atual() { return atual || 'pt'; },
    get escolhido() { return atual != null; },
    get local() { return IDIOMAS[atual || 'pt'].local; },
    get nome() { return IDIOMAS[atual || 'pt'].nome; },
    definir(novo) {
      atual = novo;
      try { localStorage.setItem('idioma', JSON.stringify(novo)); } catch (_) {}
      document.documentElement.lang = IDIOMAS[novo].local;
    },
  };
})();

const EN = {
  // Seções e barras
  'Início': 'Home',
  'Bíblia': 'Bible',
  'Biblioteca': 'Library',
  'Reuniões': 'Meetings',
  'Estudo Pessoal': 'Personal Study',
  'Menu': 'Menu',
  'Voltar': 'Back',
  'Importar publicações': 'Import publications',
  'Histórico': 'History',
  'Configurações': 'Settings',
  'Buscar na Bíblia': 'Search the Bible',
  'Buscar nas publicações': 'Search publications',
  'BÍBLIA': 'BIBLE',
  'Marcadores': 'Bookmarks',
  'Mais opções': 'More options',
  'Ordenar': 'Sort',
  'Backup e restauração': 'Backup and restore',
  'Notas e etiquetas': 'Notes and tags',
  // Geral
  'Fechar': 'Close',
  'Cancelar': 'Cancel',
  'Limpar': 'Clear',
  'Importar': 'Import',
  'Concluído': 'Done',
  'Não deu para abrir': "Couldn't open this",
  'Nada aqui.': 'Nothing here.',
  'Nada lido ainda.': 'Nothing read yet.',
  'Solte os arquivos .jwpub para importar': 'Drop .jwpub files to import',
  // Idioma
  'Idioma': 'Language',
  'Escolha o idioma do aplicativo': 'Choose the app language',
  'Dá para trocar depois em Configurações.': 'You can change it later in Settings.',
  'As publicações continuam no idioma de cada arquivo .jwpub.': 'Publications stay in the language of each .jwpub file.',
  // Configurações
  'Exibição': 'Display',
  'Aparência': 'Appearance',
  'Padrão do sistema': 'System default',
  'Claro': 'Light',
  'Escuro': 'Dark',
  'Avisar sobre arquivos .jwpub novos em Downloads': 'Notify about new .jwpub files in Downloads',
  'Sobre': 'About',
  'JW Library on Linux {v} — leitor de publicações .jwpub para Linux, feito pela comunidade. Não é um aplicativo oficial.':
    'JW Library on Linux {v} — a .jwpub publication reader for Linux, made by the community. Not an official app.',
  // Importar
  'Há um arquivo novo em Downloads: {x}': 'There is a new file in Downloads: {x}',
  'Há {n} arquivos .jwpub novos em Downloads.': 'There are {n} new .jwpub files in Downloads.',
  'Importando…': 'Importing…',
  '{n} publicação(ões) importada(s)': '{n} publication(s) imported',
  'não importado: {x}': 'not imported: {x}',
  'Nada para importar': 'Nothing to import',
  'Publicações do JW Library': 'JW Library publications',
  'Erro ao ler a biblioteca: {x}': 'Error reading the library: {x}',
  // Painel e leitor
  'Painel de estudo': 'Study panel',
  'Nota de rodapé {n}': 'Footnote {n}',
  'Nota de rodapé': 'Footnote',
  'Nota de estudo': 'Study note',
  'Nenhum conteúdo de estudo.': 'No study content.',
  'Parágrafo {n}': 'Paragraph {n}',
  'Abrir o capítulo': 'Open chapter',
  'Texto bíblico': 'Bible text',
  'Referências marginais': 'Marginal references',
  'Sem referências.': 'No references.',
  'Essa publicação não está na sua biblioteca.': 'That publication is not in your library.',
  'Link externo: {x}': 'External link: {x}',
  'Notas de estudo': 'Study notes',
  'Estudo': 'Study',
  'Adicionar uma nota': 'Add a note',
  'Nota': 'Note',
  'Copiar': 'Copy',
  'Versículo copiado.': 'Verse copied.',
  'Copiado.': 'Copied.',
  'Não deu para copiar.': "Couldn't copy.",
  'Este versículo não tem notas de estudo.': 'This verse has no study notes.',
  'Configurações de texto': 'Text settings',
  'Tela inteira': 'Full screen',
  'Esconder painel': 'Hide panel',
  'Mostrar painel': 'Show panel',
  'Capítulo anterior': 'Previous chapter',
  'Capítulo seguinte': 'Next chapter',
  'Livro anterior': 'Previous book',
  'Livro seguinte': 'Next book',
  'Anterior': 'Previous',
  'Seguinte': 'Next',
  'Carregando…': 'Loading…',
  // Bíblia
  'Nenhuma Bíblia instalada': 'No Bible installed',
  'Importe o arquivo .jwpub de uma Bíblia (por exemplo, nwtsty_T.jwpub, a Bíblia de Estudo).':
    'Import the .jwpub file of a Bible (for example, nwtsty_E.jwpub, the Study Bible).',
  'Escrituras Hebraico-Aramaicas': 'Hebrew-Aramaic Scriptures',
  'Escrituras Gregas Cristãs': 'Christian Greek Scriptures',
  'Palavras ou expressão': 'Words or phrase',
  'Achar a expressão exata': 'Find the exact phrase',
  'Buscar': 'Search',
  'Buscando… (a primeira busca prepara o índice e demora alguns segundos)': 'Searching… (the first search builds the index and takes a few seconds)',
  '{n} ocorrência': '{n} result',
  '{n} ocorrências': '{n} results',
  ' (mostrando {n})': ' (showing {n})',
  // Biblioteca
  'Remover da biblioteca': 'Remove from library',
  'Baixar publicações': 'Download publications',
  'Remover': 'Remove',
  '"{x}" sai da biblioteca deste computador. Seus grifos e notas continuam guardados e voltam se você importar a publicação de novo.':
    '"{x}" will be removed from this computer\'s library. Your highlights and notes are kept and come back if you import the publication again.',
  'Publicação removida.': 'Publication removed.',
  'Não deu para remover: {x}': "Couldn't remove: {x}",
  'Título': 'Title',
  'Maior arquivo': 'Largest file',
  'Mais recentes': 'Most recent',
  'PUBLICAÇÕES': 'PUBLICATIONS',
  'BAIXADOS': 'DOWNLOADED',
  'A biblioteca está vazia': 'The library is empty',
  'Arraste arquivos .jwpub para esta janela ou use o botão abaixo.': 'Drag .jwpub files into this window or use the button below.',
  // Início
  'Bem-vindo ao JW Library on Linux': 'Welcome to JW Library on Linux',
  '{n} publicações na sua biblioteca.': '{n} publications in your library.',
  'Importe os arquivos .jwpub que você já tem para começar a ler.': 'Import the .jwpub files you already have to start reading.',
  'Continuar lendo': 'Continue reading',
  'Mais recentes na biblioteca': 'Latest in your library',
  // Reuniões
  'Semana anterior': 'Previous week',
  'Semana seguinte': 'Next week',
  'Esta semana': 'This week',
  'Vida e Ministério': 'Life and Ministry',
  'Estudo de A Sentinela': 'Watchtower Study',
  'Importe a Apostila Vida e Ministério que cobre esta semana (arquivo mwb_….jwpub).': 'Import the Life and Ministry Meeting Workbook for this week (file mwb_….jwpub).',
  'Importe a edição de estudo de A Sentinela que cobre esta semana (arquivo w_….jwpub).': 'Import the study edition of The Watchtower for this week (file w_….jwpub).',
  // Estudo pessoal
  'Buscar nas notas': 'Search notes',
  'Apagar a etiqueta': 'Delete tag',
  'Dica: escreva etiquetas nas notas (separadas por vírgula) para agrupar por assunto.': 'Tip: add tags to your notes (separated by commas) to group them by topic.',
  '(sem texto)': '(no text)',
  'Nenhuma nota ainda': 'No notes yet',
  'Nenhuma nota encontrada': 'No notes found',
  'Selecione um trecho no leitor e escolha "Adicionar uma nota", ou clique no número de um versículo.': 'Select text in the reader and choose "Add a note", or click a verse number.',
  'Apagar etiqueta': 'Delete tag',
  'A etiqueta "{x}" sai de todas as notas. As notas continuam.': 'The tag "{x}" will be removed from all notes. The notes stay.',
  'Apagar': 'Delete',
  'A publicação desta nota não está na sua biblioteca.': "This note's publication is not in your library.",
  // Backup
  '{g} grifos, {n} notas, {e} etiquetas e {m} marcadores': '{g} highlights, {n} notes, {e} tags and {m} bookmarks',
  'Neste computador: {x}.': 'On this computer: {x}.',
  'O backup usa o mesmo formato do JW Library (.jwlibrary): dá para levar para o celular ou trazer de lá.': 'Backups use the same format as JW Library (.jwlibrary): you can move them to and from your phone.',
  'Criar backup': 'Create backup',
  'Restaurar backup': 'Restore backup',
  'Backup do JW Library': 'JW Library backup',
  'Backup criado com {x}.': 'Backup created with {x}.',
  'Não deu para criar o backup: {x}': "Couldn't create the backup: {x}",
  'Este arquivo não serve: {x}': "This file can't be used: {x}",
  'O backup tem {x}.': 'The backup has {x}.',
  'O backup de {a} ({d}) tem {x}.': 'The backup from {a} ({d}) has {x}.',
  'Ele <b>substitui</b> todo o estudo pessoal deste computador. Uma cópia do que existe agora fica guardada.': 'It <b>replaces</b> all personal study on this computer. A copy of what exists now is kept.',
  'Restaurar': 'Restore',
  'Restaurado: {x}.': 'Restored: {x}.',
  'Não deu para restaurar: {x}': "Couldn't restore: {x}",
  // Grifos, notas e marcadores
  'Amarelo': 'Yellow', 'Verde': 'Green', 'Azul': 'Blue', 'Rosa': 'Pink', 'Laranja': 'Orange', 'Roxo': 'Purple',
  'Tem nota': 'Has a note',
  'Tirar o grifo': 'Remove highlight',
  'Minhas notas': 'My notes',
  'Excluir a nota': 'Delete note',
  'Excluir nota': 'Delete note',
  'Isso não poderá ser desfeito.': "This can't be undone.",
  'Excluir': 'Delete',
  '+ Adicionar uma etiqueta (separe por vírgula)': '+ Add a tag (separate with commas)',
  'Não usado — clique para marcar aqui': 'Unused — click to bookmark here',
  'Tirar o marcador': 'Remove bookmark',
  'Marcador colocado.': 'Bookmark added.',
};

function t(pt, vars) {
  let s = Idioma.atual === 'en' ? (EN[pt] ?? pt) : pt;
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.split(`{${k}}`).join(v);
  return s;
}
