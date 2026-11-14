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

// ---------- Diálogos ----------
function dialogo(titulo, corpo, botoes = [['Fechar', null]]) {
  const f = document.createElement('div');
  f.className = 'fundo-dialogo';
  f.innerHTML = `<div class="dialogo" role="dialog"><h2>${esc(titulo)}</h2><div class="corpo">${corpo}</div><div class="rodape"></div></div>`;
  const rodape = f.querySelector('.rodape');
  for (const [nome, fn] of botoes) {
    const b = document.createElement('button');
    b.className = 'botao' + (fn ? ' leve' : '');
    b.textContent = nome;
    b.onclick = () => { if (fn) fn(); f.remove(); };
    rodape.appendChild(b);
  }
  f.addEventListener('click', (e) => { if (e.target === f) f.remove(); });
  document.body.appendChild(f);
  return f;
}
function historico() {
  const h = ler('historico', []);
  const corpo = h.length
    ? `<div class="lista">${h.map((x, i) => `<button class="item" data-h="${i}"><div><div class="t">${esc(x.titulo)}</div><div class="m">${esc(x.publicacao)}</div></div></button>`).join('')}</div>`
    : '<p class="vazio">Nada lido ainda.</p>';
  const f = dialogo('Histórico', corpo, [['Limpar', () => { guardar('historico', []); }], ['Fechar', null]]);
  f.querySelectorAll('[data-h]').forEach((b) => (b.onclick = () => {
    const x = h[+b.dataset.h];
    f.remove();
    abrir({ tela: 'documento', pasta: x.pasta, id: x.id });
  }));
}
function aplicarTema() {
  const t = ler('tema', 'sistema');
  if (t === 'sistema') document.documentElement.removeAttribute('data-tema');
  else document.documentElement.dataset.tema = t;
}
function configuracoes() {
  const t = ler('tema', 'sistema');
  const corpo = `<div class="grupo">Exibição</div>
    <label class="opcao">Aparência <select id="tema">${[['sistema', 'Padrão do sistema'], ['claro', 'Claro'], ['escuro', 'Escuro']].map(([k, n]) =>
      `<option value="${k}"${k === t ? ' selected' : ''}>${n}</option>`).join('')}</select></label>
    <label class="opcao">Avisar sobre arquivos .jwpub novos em Downloads <input type="checkbox" id="avisar"${ler('avisar_downloads', true) ? ' checked' : ''}></label>
    <div class="grupo">Sobre</div>
    <p>Leitor JW 0.1 — leitor de publicações .jwpub para Linux, feito pela comunidade. Não é um aplicativo oficial.</p>`;
  const f = dialogo('Configurações', corpo);
  f.querySelector('#tema').onchange = (e) => { guardar('tema', e.target.value); aplicarTema(); };
  f.querySelector('#avisar').onchange = (e) => { guardar('avisar_downloads', e.target.checked); desenhar(); };
}

// ---------- Arquivos novos em Downloads ----------
async function faixaNovos(c) {
  if (!ler('avisar_downloads', true)) return;
  let novos = [];
  try { novos = await chamar('novos_em_downloads'); } catch (_) { return; }
  if (!novos.length) return;
  const faixa = document.createElement('div');
  faixa.className = 'faixa-novos';
  faixa.innerHTML = `${icone('importar')}<span>${novos.length === 1 ? `Há um arquivo novo em Downloads: ${esc(novos[0].nome)}` : `Há ${novos.length} arquivos .jwpub novos em Downloads.`}</span><button class="botao">Importar</button>`;
  faixa.querySelector('button').onclick = () => importar(novos.map((n) => n.caminho));
  c.querySelector('.tela')?.prepend(faixa);
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

// ---------- Painel de estudo ----------
async function montarPainel(c, e) {
  const corpo = c.querySelector('.painel-corpo');
  let p;
  try { p = await chamar('painel', { pasta: e.pasta, id: e.id }); } catch (erro) { corpo.innerHTML = `<p class="vazio">${esc(erro)}</p>`; return; }
  const porParagrafo = new Map();
  const juntar = (par, html) => { if (!porParagrafo.has(par)) porParagrafo.set(par, []); porParagrafo.get(par).push(html); };
  for (const n of p.notas) juntar(n.paragrafo, `<div class="bloco" data-nota="${n.indice}"><div class="rotulo">Nota de rodapé ${n.indice}</div>${n.html}</div>`);
  for (const t of p.trechos) juntar(t.paragrafo, `<div class="bloco"><div class="rotulo">${esc(t.origem)}</div>${t.html}</div>`);
  if (!porParagrafo.size) { corpo.innerHTML = '<p class="vazio" style="padding:20px 0">Nenhum conteúdo de estudo.</p>'; return; }
  corpo.innerHTML = [...porParagrafo.keys()].sort((a, b) => a - b).map((par) => {
    const num = c.querySelector(`.documento [data-pid="${par}"] .parNum`)?.textContent?.replace(/\D/g, '');
    return `<div class="paragrafo" data-par="${par}">${num ? `Parágrafo ${num}` : '&nbsp;'}</div>${porParagrafo.get(par).join('')}`;
  }).join('');
  corpo.querySelectorAll('.paragrafo[data-par]').forEach((d) => (d.onclick = () => {
    const alvo = c.querySelector(`.documento [data-pid="${d.dataset.par}"]`);
    alvo?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }));
  corpo.addEventListener('click', (ev) => { const a = ev.target.closest('a'); if (a) ev.preventDefault(); });
}
function mostrarNota(c, indice) {
  if (!c.querySelector('.leitor').classList.contains('com-painel')) {
    guardar('painel', true);
    c.querySelector('.leitor').classList.add('com-painel');
    $('#flutuante').innerHTML = icone('fechar_painel');
  }
  const bloco = c.querySelector(`.painel [data-nota="${indice}"]`);
  if (!bloco) return;
  c.querySelectorAll('.painel .foco').forEach((x) => x.classList.remove('foco'));
  bloco.classList.add('foco');
  bloco.scrollIntoView({ behavior: 'smooth', block: 'center' });
}

// ---------- Telas ----------
const TELAS = {
  vazia(c) {
    c.innerHTML = '<div class="vazio">Nada aqui.</div>';
  },

  inicio(c) {
    barra('Início', 'Português (Brasil)', [['importar', 'Importar publicações', escolherArquivos], ['historico', 'Histórico', historico], ['config', 'Configurações', configuracoes]]);
    const lidos = ler('historico', []).slice(0, 8);
    const recentes = biblioteca.cartoes.slice().sort((a, b) => b.ano - a.ano || b.edicao - a.edicao).slice(0, 6);
    c.innerHTML = `<div class="tela">
      <div class="boas-vindas"><h1>Bem-vindo ao Leitor JW</h1>
        <p>${biblioteca.cartoes.length ? `${biblioteca.cartoes.length} publicações na sua biblioteca.` : 'Importe os arquivos .jwpub que você já tem para começar a ler.'}</p>
        <button class="botao" id="imp">Importar publicações</button></div>
      ${lidos.length ? `<div class="secao-titulo">Continuar lendo</div><div class="lista">${lidos.map((h, i) =>
        `<button class="item" data-h="${i}"><div><div class="t">${esc(h.titulo)}</div><div class="m">${esc(h.publicacao)}</div></div></button>`).join('')}</div>` : ''}
      ${recentes.length ? `<div class="secao-titulo">Mais recentes na biblioteca</div><div class="grade">${recentes.map((r) => cartao(r)).join('')}</div>` : ''}
    </div>`;
    $('#imp').onclick = escolherArquivos;
    c.querySelectorAll('[data-h]').forEach((b) => (b.onclick = () => {
      const h = lidos[+b.dataset.h];
      abrir({ tela: 'documento', pasta: h.pasta, id: h.id });
    }));
    ligarCartoes(c);
    faixaNovos(c);
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
    const acoes = [['importar', 'Importar publicações', escolherArquivos], ['historico', 'Histórico', historico]];
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
    faixaNovos(c);
  },

  categoria(c, e) {
    const k = biblioteca.categorias.find((x) => x.chave === e.chave);
    const daqui = biblioteca.cartoes.filter((x) => x.categoria === e.chave).sort((a, b) => b.ano - a.ano || b.edicao - a.edicao);
    const anos = [...new Set(daqui.map((x) => x.ano))];
    const ano = e.ano ?? (anos.length === 1 ? anos[0] : null);
    barra(k ? k.nome : 'Biblioteca', ano && anos.length > 1 ? String(ano) : 'Português (Brasil)', [['historico', 'Histórico', historico]]);
    if (ano == null) {
      c.innerHTML = `<div class="tela"><div class="grade">${anos.map((a) =>
        `<button class="categoria ano" data-ano="${a}">${a}</button>`).join('')}</div></div>`;
      c.querySelectorAll('[data-ano]').forEach((b) => (b.onclick = () => abrir({ tela: 'categoria', chave: e.chave, ano: +b.dataset.ano })));
      return;
    }
    c.innerHTML = `<div class="tela"><div class="grade">${daqui.filter((x) => x.ano === ano).map((x) => cartao(x)).join('')}</div></div>`;
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
    const textoConfig = (b) => {
      const m = menu(b, `<div class="deslizante"><small>A</small><input type="range" min="13" max="30" value="${ler('tamanho', 18)}"><big>A</big></div>`);
      m.querySelector('input').oninput = (x) => { guardar('tamanho', +x.target.value); $('.documento').style.setProperty('--tam', x.target.value + 'px'); };
    };
    barra(d.titulo, d.publicacao, [
      ['historico', 'Histórico', historico],
      ['mais', 'Mais opções', (ev, b) => {
        const m = menu(b, '<button data-o="texto">Configurações de texto</button><button data-o="tela">Tela inteira</button>');
        m.onclick = async (x) => {
          const o = x.target.dataset.o;
          if (o === 'texto') textoConfig(b);
          if (o === 'tela') { fecharMenu(); const j = tauri.window.getCurrentWindow(); j.setFullscreen(!(await j.isFullscreen())); }
        };
      }],
    ]);
    c.innerHTML = `<div class="leitor${comPainel ? ' com-painel' : ''}">
      <div class="coluna"><article class="documento" style="--tam:${tam}px">${d.html}</article></div>
      <aside class="painel"><h3>Painel de estudo</h3><div class="painel-corpo"><p class="vazio" style="padding:20px 0">Carregando…</p></div></aside>
    </div>
    ${d.anterior ? `<button class="seta esq" title="Anterior">${icone('esq')}</button>` : ''}
    ${d.seguinte ? `<button class="seta dir" title="Seguinte">${icone('dir')}</button>` : ''}
    <button id="flutuante" title="${comPainel ? 'Esconder painel' : 'Mostrar painel'}">${icone(comPainel ? 'fechar_painel' : 'painel')}</button>`;
    montarPainel(c, e);
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
      } else if (a.dataset.fnid) {
        ev.preventDefault();
        mostrarNota(c, a.dataset.fnid);
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

  async reunioes(c, e) {
    barra('Reuniões', 'Português (Brasil)', [['historico', 'Histórico', historico]]);
    const hoje = new Date();
    const seg = e.segunda ? new Date(e.segunda) : new Date(hoje.getFullYear(), hoje.getMonth(), hoje.getDate() - ((hoje.getDay() + 6) % 7));
    const dom = new Date(seg.getFullYear(), seg.getMonth(), seg.getDate() + 6);
    const n = (d) => d.getFullYear() * 10000 + (d.getMonth() + 1) * 100 + d.getDate();
    const mes = (d) => d.toLocaleDateString('pt-BR', { month: 'long' });
    const rotulo = seg.getMonth() === dom.getMonth() ? `${seg.getDate()}-${dom.getDate()} de ${mes(dom)}` : `${seg.getDate()} de ${mes(seg)} – ${dom.getDate()} de ${mes(dom)}`;
    const atual = n(hoje) >= n(seg) && n(hoje) <= n(dom);
    const s = await chamar('semana', { data: n(seg) });
    const parte = (titulo, x, falta) => `<div class="reuniao"><h3>${titulo}</h3>${x
      ? `<button class="item" data-pasta="${esc(x.pasta)}" data-id="${x.id}">${x.miniatura ? `<img src="${base}/midia/${encodeURIComponent(x.pasta)}/${encodeURIComponent(x.miniatura)}" alt="">` : ''}<div><div class="t">${esc(x.titulo)}</div><div class="m">${esc(x.publicacao)}</div></div></button>`
      : `<p class="falta">${falta}</p>`}</div>`;
    c.innerHTML = `<div class="tela">
      <div class="semana-nav"><button class="seta-s" data-mover="-7" title="Semana anterior">${icone('esq')}</button>
        <div class="data">${rotulo}<small>${atual ? 'Esta semana' : seg.getFullYear()}</small></div>
        <button class="seta-s" data-mover="7" title="Semana seguinte">${icone('dir')}</button></div>
      ${parte('Vida e Ministério', s.apostila, 'Importe a Apostila Vida e Ministério que cobre esta semana (arquivo mwb_….jwpub).')}
      ${parte('Estudo de A Sentinela', s.sentinela, 'Importe a edição de estudo de A Sentinela que cobre esta semana (arquivo w_….jwpub).')}
    </div>`;
    c.querySelectorAll('[data-mover]').forEach((b) => (b.onclick = () => {
      const d = new Date(seg.getFullYear(), seg.getMonth(), seg.getDate() + +b.dataset.mover);
      e.segunda = d.toISOString();
      desenhar();
    }));
    c.querySelectorAll('.item[data-pasta]').forEach((b) => (b.onclick = () => abrir({ tela: 'documento', pasta: b.dataset.pasta, id: +b.dataset.id })));
  },

  estudo(c) {
    barra('Estudo Pessoal', 'Português (Brasil)');
    c.innerHTML = '<div class="vazio"><h2>Estudo Pessoal</h2><p>Grifos, notas, etiquetas e marcadores chegam na Etapa 4.</p></div>';
  },
};

(async () => {
  aplicarTema();
  montarTrilho();
  try { await carregarBiblioteca(); } catch (e) { aviso('Erro ao ler a biblioteca: ' + e); }
  ouvirArrastar();
  desenhar();
})();
