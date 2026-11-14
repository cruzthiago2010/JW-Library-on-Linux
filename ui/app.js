// Leitor JW — a moldura: trilho de 5 seções, cada uma com a sua pilha de telas
// (como no JW Library do Windows: voltar à seção devolve onde se parou; a seta ← sobe um nível).
const tauri = window.__TAURI__;
const chamar = (cmd, args) => tauri.core.invoke(cmd, args);

const SECOES = [
  ['inicio', 'Início', 'inicio'],
  ['biblia', 'Bíblia', 'biblia'],
  ['biblioteca', 'Biblioteca', 'biblioteca'],
  ['reunioes', 'Reuniões', 'reunioes'],
  ['estudo', 'Estudo Pessoal', 'estudo'],
];
const pilhas = Object.fromEntries(SECOES.map(([s]) => [s, [{ tela: s }]]));
let secao = 'inicio';
let base = '';
let biblioteca = { categorias: [], cartoes: [] };

const $ = (s, r = document) => r.querySelector(s);
const esc = (t) => String(t ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const guardar = (k, v) => { try { localStorage.setItem(k, JSON.stringify(v)); } catch (_) {} };
const ler = (k, padrao) => { try { return JSON.parse(localStorage.getItem(k)) ?? padrao; } catch (_) { return padrao; } };

function aviso(texto, ms = 3500) {
  const a = $('#aviso');
  a.textContent = texto;
  a.style.display = 'block';
  clearTimeout(aviso.t);
  aviso.t = setTimeout(() => (a.style.display = 'none'), ms);
}

// ---------- Trilho e barras ----------
function montarTrilho() {
  const t = $('#trilho');
  t.innerHTML = `<button data-menu title="Menu">${icone('menu')}<span class="rotulo"></span></button>` +
    SECOES.map(([s, nome, ic]) => `<button data-secao="${s}" title="${nome}">${icone(ic)}<span class="rotulo">${nome}</span></button>`).join('');
  t.addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.hasAttribute('data-menu')) return t.classList.toggle('aberto');
    t.classList.remove('aberto');
    irPara(b.dataset.secao);
  });
  $('#voltar').innerHTML = icone('voltar');
  $('#voltar').onclick = voltar;
}

function irPara(s) {
  secao = s;
  desenhar();
}
function abrir(entrada) {
  pilhas[secao].push(entrada);
  desenhar();
}
function voltar() {
  if (pilhas[secao].length > 1) {
    pilhas[secao].pop();
    desenhar();
  }
}
document.addEventListener('keydown', (e) => {
  if (e.altKey && e.key === 'ArrowLeft') voltar();
  if (e.key === 'Escape') fecharMenu();
});
document.addEventListener('mouseup', (e) => { if (e.button === 3) voltar(); });

function barra(nome, sub, acoes = []) {
  $('#cabecalho .nome').textContent = nome;
  $('#cabecalho .sub').textContent = sub || '';
  const a = $('#acoes');
  a.innerHTML = '';
  for (const [ic, titulo, fn] of acoes) {
    const b = document.createElement('button');
    b.title = titulo;
    b.innerHTML = icone(ic);
    b.onclick = (e) => fn(e, b);
    a.appendChild(b);
  }
}

async function desenhar() {
  document.querySelectorAll('#trilho [data-secao]').forEach((b) => b.classList.toggle('ativo', b.dataset.secao === secao));
  const pilha = pilhas[secao];
  $('#voltar').classList.toggle('ativo', pilha.length > 1);
  const entrada = pilha[pilha.length - 1];
  const c = $('#conteudo');
  fecharMenu();
  try {
    await (TELAS[entrada.tela] || TELAS.vazia)(c, entrada);
  } catch (erro) {
    c.innerHTML = `<div class="vazio"><h2>Não deu para abrir</h2><p>${esc(erro)}</p></div>`;
  }
  if (entrada.rolagem != null) c.scrollTop = entrada.rolagem;
  else c.scrollTop = 0;
}
$('#conteudo').addEventListener('scroll', () => {
  const p = pilhas[secao];
  p[p.length - 1].rolagem = $('#conteudo').scrollTop;
});

// ---------- Menus ----------
function fecharMenu() {
  document.querySelectorAll('.menu').forEach((m) => m.remove());
}
function menu(botao, conteudo) {
  fecharMenu();
  const m = document.createElement('div');
  m.className = 'menu';
  m.innerHTML = conteudo;
  document.body.appendChild(m);
  const r = botao.getBoundingClientRect();
  m.style.top = r.bottom + 4 + 'px';
  m.style.left = Math.max(8, r.right - m.offsetWidth) + 'px';
  setTimeout(() => document.addEventListener('click', function fora(e) {
    if (!m.contains(e.target)) { m.remove(); document.removeEventListener('click', fora); }
  }), 0);
  return m;
}

// ---------- Importar ----------
async function importar(caminhos) {
  if (!caminhos || !caminhos.length) return;
  aviso('Importando…', 60000);
  const r = await chamar('importar', { caminhos });
  await carregarBiblioteca();
  const partes = [];
  if (r.novas.length) partes.push(`${r.novas.length} publicação(ões) importada(s)`);
  if (r.erros.length) partes.push(`não importado: ${r.erros.join('; ')}`);
  aviso(partes.join(' · ') || 'Nada para importar', 6000);
  desenhar();
}
async function escolherArquivos() {
  const escolhidos = await tauri.dialog.open({
    multiple: true,
    title: 'Importar publicações',
    filters: [{ name: 'Publicações do JW Library', extensions: ['jwpub'] }],
  });
  if (escolhidos) importar(Array.isArray(escolhidos) ? escolhidos : [escolhidos]);
}
async function ouvirArrastar() {
  const janela = tauri.webviewWindow?.getCurrentWebviewWindow?.() || tauri.webview?.getCurrentWebview?.();
  if (!janela?.onDragDropEvent) return;
  await janela.onDragDropEvent((ev) => {
    const tipo = ev.payload.type;
    document.body.classList.toggle('soltando', tipo === 'enter' || tipo === 'over');
    if (tipo === 'drop') importar(ev.payload.paths.filter((p) => p.toLowerCase().endsWith('.jwpub')));
  });
}

async function carregarBiblioteca() {
  biblioteca = await chamar('listar');
  base = biblioteca.base;
}
const capa = (c) => (c.capa ? `${base}/midia/${encodeURIComponent(c.pasta)}/${encodeURIComponent(c.capa)}` : '');
const megas = (b) => (b / 1048576).toFixed(b > 10485760 ? 0 : 1).replace('.', ',') + ' MB';
const ICONE_CATEGORIA = { biblia: 'biblia', livros: 'livro', brochuras: 'brochura', folhetos: 'folheto', sentinela: 'revista', despertai: 'revista', apostilas: 'apostila' };

function cartao(c, extra = '') {
  const img = c.capa ? `<img src="${capa(c)}" alt="" loading="lazy">` : '<div class="semcapa"></div>';
  return `<button class="cartao" data-pasta="${esc(c.pasta)}">${img}<div><div class="t">${esc(c.titulo)}</div><div class="m">${extra || c.ano}</div></div></button>`;
}
function ligarCartoes(raiz) {
  raiz.querySelectorAll('.cartao[data-pasta]').forEach((b) => (b.onclick = () => abrir({ tela: 'sumario', pasta: b.dataset.pasta })));
}

// ---------- Telas ----------
const TELAS = {
  vazia(c) {
    c.innerHTML = '<div class="vazio">Nada aqui.</div>';
  },

  inicio(c) {
    barra('Início', 'Português (Brasil)', [['importar', 'Importar publicações', escolherArquivos]]);
    const historico = ler('historico', []).slice(0, 8);
    const recentes = biblioteca.cartoes.slice().sort((a, b) => b.ano - a.ano || b.edicao - a.edicao).slice(0, 6);
    c.innerHTML = `<div class="tela">
      <div class="boas-vindas"><h1>Bem-vindo ao Leitor JW</h1>
        <p>${biblioteca.cartoes.length ? `${biblioteca.cartoes.length} publicações na sua biblioteca.` : 'Importe os arquivos .jwpub que você já tem para começar a ler.'}</p>
        <button class="botao" id="imp">Importar publicações</button></div>
      ${historico.length ? `<div class="secao-titulo">Continuar lendo</div><div class="lista">${historico.map((h, i) =>
        `<button class="item" data-h="${i}"><div><div class="t">${esc(h.titulo)}</div><div class="m">${esc(h.publicacao)}</div></div></button>`).join('')}</div>` : ''}
      ${recentes.length ? `<div class="secao-titulo">Mais recentes na biblioteca</div><div class="grade">${recentes.map((r) => cartao(r)).join('')}</div>` : ''}
    </div>`;
    $('#imp').onclick = escolherArquivos;
    c.querySelectorAll('[data-h]').forEach((b) => (b.onclick = () => {
      const h = historico[+b.dataset.h];
      abrir({ tela: 'documento', pasta: h.pasta, id: h.id });
    }));
    ligarCartoes(c);
  },

  biblia(c) {
    barra('Bíblia', 'Português (Brasil)');
    const b = biblioteca.cartoes.find((x) => x.categoria === 'biblia');
    if (b) {
      c.innerHTML = `<div class="tela"><div class="secao-titulo">Bíblias instaladas</div><div class="grade">${biblioteca.cartoes.filter((x) => x.categoria === 'biblia').map((x) => cartao(x)).join('')}</div>
        <p class="vazio">A grade de livros e capítulos chega na Etapa 3. Por enquanto a Bíblia abre pelo sumário.</p></div>`;
      ligarCartoes(c);
    } else {
      c.innerHTML = '<div class="vazio"><h2>Nenhuma Bíblia instalada</h2><p>Importe o arquivo .jwpub de uma Bíblia (por exemplo, nwtsty_T.jwpub).</p><button class="botao" id="imp">Importar</button></div>';
      $('#imp').onclick = escolherArquivos;
    }
  },

  biblioteca(c, e) {
    const aba = e.aba || 'publicacoes';
    const acoes = [['importar', 'Importar publicações', escolherArquivos]];
    if (aba === 'baixados') acoes.unshift(['ordenar', 'Ordenar', (ev, botao) => {
      const m = menu(botao, ['titulo:Título', 'maior:Maior arquivo', 'recente:Mais recentes'].map((o) => {
        const [k, n] = o.split(':');
        return `<button data-o="${k}">${(e.ordem || 'titulo') === k ? '✓ ' : ''}${n}</button>`;
      }).join(''));
      m.onclick = (x) => { const o = x.target.dataset.o; if (o) { e.ordem = o; desenhar(); } };
    }]);
    barra('Biblioteca', 'Português (Brasil)', acoes);
    const abas = `<div class="abas">${[['publicacoes', 'PUBLICAÇÕES'], ['baixados', 'BAIXADOS']].map(([k, n]) =>
      `<button data-aba="${k}" class="${k === aba ? 'ativa' : ''}">${n}</button>`).join('')}</div>`;
    if (!biblioteca.cartoes.length) {
      c.innerHTML = `<div class="tela">${abas}<div class="vazio"><h2>A biblioteca está vazia</h2><p>Arraste arquivos .jwpub para esta janela ou use o botão abaixo.</p><button class="botao" id="imp">Importar publicações</button></div></div>`;
      $('#imp').onclick = escolherArquivos;
    } else if (aba === 'publicacoes') {
      c.innerHTML = `<div class="tela">${abas}<div class="grade">${biblioteca.categorias.map((k) =>
        `<button class="categoria" data-cat="${k.chave}">${icone(ICONE_CATEGORIA[k.chave] || 'livro')}<span>${esc(k.nome)}</span><span class="quantas">${k.quantas}</span></button>`).join('')}</div></div>`;
      c.querySelectorAll('[data-cat]').forEach((b) => (b.onclick = () => abrir({ tela: 'categoria', chave: b.dataset.cat })));
    } else {
      const ordem = e.ordem || 'titulo';
      const lista = biblioteca.cartoes.slice().sort((a, b) =>
        ordem === 'maior' ? b.tamanho - a.tamanho : ordem === 'recente' ? b.ano - a.ano || b.edicao - a.edicao : a.titulo.localeCompare(b.titulo, 'pt'));
      c.innerHTML = `<div class="tela">${abas}${biblioteca.categorias.map((k) => {
        const daqui = lista.filter((x) => x.categoria === k.chave);
        return `<div class="grupo">${esc(k.nome)}</div><div class="grade">${daqui.map((x) => cartao(x, `${x.ano} • ${megas(x.tamanho)}`)).join('')}</div>`;
      }).join('')}</div>`;
      ligarCartoes(c);
    }
    c.querySelectorAll('[data-aba]').forEach((b) => (b.onclick = () => { e.aba = b.dataset.aba; desenhar(); }));
  },

  categoria(c, e) {
    const k = biblioteca.categorias.find((x) => x.chave === e.chave);
    barra(k ? k.nome : 'Biblioteca', 'Português (Brasil)');
    const daqui = biblioteca.cartoes.filter((x) => x.categoria === e.chave).sort((a, b) => b.ano - a.ano || b.edicao - a.edicao);
    const anos = [...new Set(daqui.map((x) => x.ano))];
    c.innerHTML = `<div class="tela">${anos.map((a) =>
      `<div class="grupo">${a}</div><div class="grade">${daqui.filter((x) => x.ano === a).map((x) => cartao(x)).join('')}</div>`).join('')}</div>`;
    ligarCartoes(c);
  },

  async sumario(c, e) {
    const s = await chamar('sumario', { pasta: e.pasta });
    barra(s.titulo, 'Português (Brasil)');
    const img = (m) => (m ? `<img src="${base}/midia/${encodeURIComponent(s.pasta)}/${encodeURIComponent(m)}" alt="" loading="lazy">` : '');
    const itens = (lista) => lista.map((i) => {
      if (!i.documento && i.filhos.length) return `<div class="item secao">${esc(i.titulo)}</div>${itens(i.filhos)}`;
      return `<button class="item" data-doc="${i.documento ?? ''}">${img(i.miniatura)}<span class="t">${esc(i.titulo)}</span></button>${i.filhos.length ? `<div class="filhos">${itens(i.filhos)}</div>` : ''}`;
    }).join('');
    const c0 = biblioteca.cartoes.find((x) => x.pasta === s.pasta);
    c.innerHTML = `<div class="tela"><div class="capa-grande">${s.capa ? `<img src="${capa(c0 || s)}" alt="">` : ''}<h1>${esc(s.titulo)}</h1></div><div class="lista">${itens(s.itens)}</div></div>`;
    c.querySelectorAll('[data-doc]').forEach((b) => b.dataset.doc && (b.onclick = () => abrir({ tela: 'documento', pasta: s.pasta, id: +b.dataset.doc })));
  },

  async documento(c, e) {
    const d = await chamar('documento', { pasta: e.pasta, id: e.id });
    const tam = ler('tamanho', 18);
    const comPainel = ler('painel', false);
    barra(d.titulo, d.publicacao, [
      ['texto', 'Configurações de texto', (ev, b) => {
        const m = menu(b, `<div class="deslizante"><small>A</small><input type="range" min="13" max="30" value="${ler('tamanho', 18)}"><big>A</big></div>`);
        m.querySelector('input').oninput = (x) => { guardar('tamanho', +x.target.value); $('.documento').style.setProperty('--tam', x.target.value + 'px'); };
      }],
    ]);
    c.innerHTML = `<div class="leitor${comPainel ? ' com-painel' : ''}">
      <div class="coluna"><article class="documento" style="--tam:${tam}px">${d.html}</article></div>
      <aside class="painel"><h3>Painel de estudo</h3><p class="vazio" style="padding:20px 0">Notas de rodapé, referências e notas pessoais aparecem aqui (Etapas 2 a 4).</p></aside>
    </div>
    ${d.anterior ? `<button class="seta esq" title="Anterior">${icone('esq')}</button>` : ''}
    ${d.seguinte ? `<button class="seta dir" title="Seguinte">${icone('dir')}</button>` : ''}
    <button id="flutuante" title="${comPainel ? 'Esconder painel' : 'Mostrar painel'}">${icone(comPainel ? 'fechar_painel' : 'painel')}</button>`;
    const trocar = (id) => { pilhas[secao][pilhas[secao].length - 1] = { tela: 'documento', pasta: e.pasta, id }; desenhar(); };
    c.querySelector('.seta.esq')?.addEventListener('click', () => trocar(d.anterior));
    c.querySelector('.seta.dir')?.addEventListener('click', () => trocar(d.seguinte));
    $('#flutuante').onclick = () => { guardar('painel', !comPainel); desenhar(); };
    c.querySelector('.documento').addEventListener('click', async (ev) => {
      const a = ev.target.closest('a');
      if (!a) return;
      if (a.dataset.meps) {
        ev.preventDefault();
        const achado = await chamar('achar_meps', { meps: +a.dataset.meps });
        if (achado) abrir({ tela: 'documento', pasta: achado[0], id: achado[1] });
        else aviso('Essa publicação não está na sua biblioteca.');
      } else if (a.dataset.biblia) {
        ev.preventDefault();
        aviso('Os textos bíblicos abrem no painel a partir da Etapa 3.');
      } else if (a.dataset.fora) {
        ev.preventDefault();
        aviso('Link externo: ' + a.dataset.fora, 5000);
      }
    });
    // Histórico de leitura (os últimos 30 documentos).
    const h = ler('historico', []).filter((x) => !(x.pasta === e.pasta && x.id === e.id));
    h.unshift({ pasta: e.pasta, id: e.id, titulo: d.titulo, publicacao: d.publicacao });
    guardar('historico', h.slice(0, 30));
  },

  reunioes(c) {
    barra('Reuniões', 'Português (Brasil)');
    c.innerHTML = '<div class="vazio"><h2>Reuniões</h2><p>O seletor de semana com a apostila e A Sentinela chega na Etapa 2.</p></div>';
  },

  estudo(c) {
    barra('Estudo Pessoal', 'Português (Brasil)');
    c.innerHTML = '<div class="vazio"><h2>Estudo Pessoal</h2><p>Grifos, notas, etiquetas e marcadores chegam na Etapa 4.</p></div>';
  },
};

(async () => {
  montarTrilho();
  try { await carregarBiblioteca(); } catch (e) { aviso('Erro ao ler a biblioteca: ' + e); }
  ouvirArrastar();
  desenhar();
})();
