// JW Library on Linux — a moldura: trilho de 5 seções, cada uma com a sua pilha de telas
// (como no JW Library do Windows: voltar à seção devolve onde se parou; a seta ← sobe um nível).
const tauri = window.__TAURI__;
const chamar = (cmd, args) => tauri.core.invoke(cmd, args);
const VERSAO = '0.4';

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
let temBaixador = 'nao';
const URL_BAIXADOR = 'https://github.com/cruzthiago2010/JW-Library-Downloader-on-Linux';

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
  const trilho = $('#trilho');
  trilho.innerHTML = `<button data-menu title="${t('Menu')}">${icone('menu')}<span class="rotulo"></span></button>` +
    SECOES.map(([s, nome, ic]) => `<button data-secao="${s}" title="${t(nome)}">${icone(ic)}<span class="rotulo">${t(nome)}</span></button>`).join('');
  trilho.addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.hasAttribute('data-menu')) return trilho.classList.toggle('aberto');
    trilho.classList.remove('aberto');
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
    c.innerHTML = `<div class="vazio"><h2>${t('Não deu para abrir')}</h2><p>${esc(erro)}</p></div>`;
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
function dialogo(titulo, corpo, botoes = [[t('Fechar'), null]]) {
  const f = document.createElement('div');
  f.className = 'fundo-dialogo';
  f.innerHTML = `<div class="dialogo" role="dialog"><h2>${esc(titulo)}</h2><div class="corpo">${corpo}</div><div class="rodape"></div></div>`;
  const rodape = f.querySelector('.rodape');
  for (const [nome, fn] of botoes) {
    const b = document.createElement('button');
    b.className = 'botao' + (fn || botoes.length === 1 ? '' : ' leve');
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
    : `<p class="vazio">${t('Nada lido ainda.')}</p>`;
  const f = dialogo(t('Histórico'), corpo, [[t('Limpar'), () => { guardar('historico', []); }], [t('Fechar'), null]]);
  f.querySelectorAll('[data-h]').forEach((b) => (b.onclick = () => {
    const x = h[+b.dataset.h];
    f.remove();
    abrirDoHistorico(x);
  }));
}
function aplicarTema() {
  const t = ler('tema', 'sistema');
  if (t === 'sistema') document.documentElement.removeAttribute('data-tema');
  else document.documentElement.dataset.tema = t;
}
function configuracoes() {
  const tema = ler('tema', 'sistema');
  const corpo = `<div class="grupo">${t('Exibição')}</div>
    <label class="opcao">${t('Idioma')} <select id="idioma">${Object.entries(IDIOMAS).map(([k, x]) =>
      `<option value="${k}"${k === Idioma.atual ? ' selected' : ''}>${x.nome}</option>`).join('')}</select></label>
    <label class="opcao">${t('Aparência')} <select id="tema">${[['sistema', 'Padrão do sistema'], ['claro', 'Claro'], ['escuro', 'Escuro']].map(([k, n]) =>
      `<option value="${k}"${k === tema ? ' selected' : ''}>${t(n)}</option>`).join('')}</select></label>
    <label class="opcao">${t('Avisar sobre arquivos .jwpub novos em Downloads')} <input type="checkbox" id="avisar"${ler('avisar_downloads', true) ? ' checked' : ''}></label>
    <div class="grupo">${t('Sobre')}</div>
    <p>${t('JW Library on Linux {v} — leitor de publicações .jwpub para Linux, feito pela comunidade. Não é um aplicativo oficial.', { v: VERSAO })}</p>`;
  const f = dialogo(t('Configurações'), corpo);
  f.querySelector('#idioma').onchange = (e) => trocarIdioma(e.target.value);
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
  faixa.innerHTML = `${icone('importar')}<span>${novos.length === 1 ? t('Há um arquivo novo em Downloads: {x}', { x: esc(novos[0].nome) }) : t('Há {n} arquivos .jwpub novos em Downloads.', { n: novos.length })}</span><button class="botao">${t('Importar')}</button>`;
  faixa.querySelector('button').onclick = () => importar(novos.map((n) => n.caminho));
  c.querySelector('.tela')?.prepend(faixa);
}

// ---------- Importar ----------
async function importar(caminhos) {
  if (!caminhos || !caminhos.length) return;
  aviso(t('Importando…'), 60000);
  const r = await chamar('importar', { caminhos });
  await carregarBiblioteca();
  const partes = [];
  if (r.novas.length) partes.push(t('{n} publicação(ões) importada(s)', { n: r.novas.length }));
  if (r.erros.length) partes.push(t('não importado: {x}', { x: r.erros.join('; ') }));
  aviso(partes.join(' · ') || t('Nada para importar'), 6000);
  desenhar();
}
async function escolherArquivos() {
  const escolhidos = await tauri.dialog.open({
    multiple: true,
    title: t('Importar publicações'),
    filters: [{ name: t('Publicações do JW Library'), extensions: ['jwpub'] }],
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

// O "..." de uma publicação, como no JW Library: Abrir e Excluir.
function menuDaPublicacao(ancora, pasta) {
  const c = biblioteca.cartoes.find((x) => x.pasta === pasta);
  const m = menu(ancora, `<button data-o="abrir">${t('Abrir')}</button><button data-o="excluir">${t('Excluir')}</button>`);
  m.onclick = (ev) => {
    const o = ev.target.closest('[data-o]')?.dataset.o;
    fecharMenu();
    if (o === 'abrir') abrir({ tela: 'sumario', pasta });
    if (o === 'excluir') removerPublicacao(pasta, c ? c.titulo : pasta, c ? c.tamanho : 0);
  };
}

function removerPublicacao(pasta, titulo, tamanho = 0) {
  dialogo(t('Excluir publicação'), `<p>${t('Excluir "{x}" deste computador?', { x: esc(titulo) })}${tamanho ? ` (${megas(tamanho)})` : ''}</p><p>${t('Seus grifos e notas continuam guardados e voltam se você importar a publicação de novo.')}</p>`,
    [[t('Cancelar'), null], [t('Excluir'), async () => {
      try {
        await chamar('remover_publicacao', { pasta });
        await carregarBiblioteca();
        // Some de todas as pilhas o que era desta publicação.
        for (const k of Object.keys(pilhas)) {
          const p = pilhas[k].filter((x) => x.pasta !== pasta);
          pilhas[k] = p.length ? p : [{ tela: k }];
        }
        guardar('historico', ler('historico', []).filter((x) => x.pasta !== pasta));
        aviso(t('Publicação excluída.'));
        desenhar();
      } catch (erro) { aviso(t('Não deu para excluir: {x}', { x: erro }), 8000); }
    }]]);
}

// O baixador (projeto irmão) busca e baixa as publicações do jw.org e as instala
// na mesma Biblioteca. Instalado: abre. Senão: explica e leva à página dele.
function baixarPublicacoes() {
  if (temBaixador === 'instalado') return chamar('abrir_baixador').catch((x) => aviso(String(x)));
  const corpo = `<p>${t('O Baixador para JW Library on Linux escolhe as publicações e o idioma, baixa os arquivos .jwpub do jw.org e os coloca direto na sua Biblioteca. Ele também avisa quando sai uma edição nova ou uma atualização.')}</p>
    <p>${temBaixador === 'flatpak' ? t('Depois de instalado, abra o Baixador pelo menu de aplicativos. As publicações aparecem aqui sozinhas.') : t('Depois de instalado, este botão abre o Baixador direto.')}</p>`;
  dialogo(t('Baixar publicações'), corpo, [[t('Agora não'), null], [t('Conhecer o Baixador'), abrirPaginaBaixador]]);
}
function abrirPaginaBaixador() {
  const abrirUrl = tauri.opener?.openUrl;
  if (abrirUrl) abrirUrl(URL_BAIXADOR).catch(() => aviso(URL_BAIXADOR, 10000));
  else aviso(URL_BAIXADOR, 10000);
}
function faixaBaixador(c) {
  if (temBaixador === 'instalado' || ler('aviso_baixador_fechado', false)) return;
  const f = document.createElement('div');
  f.className = 'faixa-baixador';
  f.innerHTML = `${icone('nuvem')}<span>${t('Busque e baixe publicações do jw.org com o Baixador para JW Library on Linux.')}</span>
    <button class="botao" data-o="ver">${t('Conhecer o Baixador')}</button><button class="fechar-faixa" data-o="fechar" title="${t('Fechar')}">×</button>`;
  f.onclick = (e) => {
    const o = e.target.closest('[data-o]')?.dataset.o;
    if (o === 'ver') abrirPaginaBaixador();
    if (o === 'fechar') { guardar('aviso_baixador_fechado', true); f.remove(); }
  };
  c.querySelector('.boas-vindas')?.after(f);
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
  return `<button class="cartao" data-pasta="${esc(c.pasta)}">${img}<div><div class="t">${esc(c.titulo)}</div><div class="m">${extra || c.ano}</div></div><span class="mais-cartao" role="button" title="${t('Mais opções')}" data-mais="${esc(c.pasta)}">${icone('mais')}</span></button>`;
}
function ligarCartoes(raiz) {
  raiz.querySelectorAll('.cartao[data-pasta]').forEach((b) => (b.onclick = (ev) => {
    const mais = ev.target.closest('[data-mais]');
    if (!mais) return abrir({ tela: 'sumario', pasta: b.dataset.pasta });
    ev.stopPropagation();
    menuDaPublicacao(mais, b.dataset.pasta);
  }));
}

// ---------- Painel de estudo ----------
async function montarPainel(c, e) {
  const corpo = c.querySelector('.painel-corpo');
  let p;
  try { p = await chamar('painel', { pasta: e.pasta, id: e.id }); } catch (erro) { corpo.innerHTML = `<p class="vazio">${esc(erro)}</p>`; return; }
  const porParagrafo = new Map();
  const juntar = (par, html) => { if (!porParagrafo.has(par)) porParagrafo.set(par, []); porParagrafo.get(par).push(html); };
  for (const n of p.notas) juntar(n.paragrafo, `<div class="bloco" data-nota="${n.indice}"><div class="rotulo">${t('Nota de rodapé {n}', { n: n.indice })}</div>${n.html}</div>`);
  for (const t of p.trechos) juntar(t.paragrafo, `<div class="bloco"><div class="rotulo">${esc(t.origem)}</div>${t.html}</div>`);
  if (!porParagrafo.size) { corpo.innerHTML = `<p class="vazio" style="padding:20px 0">${t('Nenhum conteúdo de estudo.')}</p>`; return; }
  corpo.innerHTML = [...porParagrafo.keys()].sort((a, b) => a - b).map((par) => {
    const num = c.querySelector(`.documento [data-pid="${par}"] .parNum`)?.textContent?.replace(/\D/g, '');
    return `<div class="paragrafo" data-par="${par}">${num ? t('Parágrafo {n}', { n: num }) : '&nbsp;'}</div>${porParagrafo.get(par).join('')}`;
  }).join('');
  corpo.querySelectorAll('.paragrafo[data-par]').forEach((d) => (d.onclick = () => {
    const alvo = c.querySelector(`.documento [data-pid="${d.dataset.par}"]`);
    alvo?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }));
  corpo.addEventListener('click', (ev) => { const a = ev.target.closest('a'); if (a) ev.preventDefault(); });
}
function mostrarNota(c, indice) {
  abrirPainel(c);
  const bloco = c.querySelector(`.painel [data-nota="${indice}"]`);
  if (!bloco) return;
  c.querySelectorAll('.painel .foco').forEach((x) => x.classList.remove('foco'));
  bloco.classList.add('foco');
  bloco.scrollIntoView({ behavior: 'smooth', block: 'center' });
}

// ---------- Bíblia: painel, versículo e links ----------
const blocoTexto = (r) => `<div class="bloco texto-biblico"><div class="rotulo">${esc(r.rotulo)}</div><div class="versos">${r.html}</div>
  <button class="link-capitulo" data-abrir="${r.inicio}">${t('Abrir o capítulo')}</button></div>`;
function abrirPainel(c) {
  const l = c.querySelector('.leitor');
  if (!l.classList.contains('com-painel')) {
    guardar('painel', true);
    l.classList.add('com-painel');
    $('#flutuante').innerHTML = icone('fechar_painel');
  }
}
function mostrarNoTopo(c, titulo, html) {
  abrirPainel(c);
  const topo = c.querySelector('.painel-topo');
  topo.innerHTML = `<div class="topo-cab"><span>${esc(titulo)}</span><button title="${t('Fechar')}">×</button></div>${html}`;
  topo.querySelector('.topo-cab button').onclick = () => (topo.innerHTML = '');
  topo.querySelectorAll('[data-abrir]').forEach((b) => (b.onclick = () => {
    const k = +b.dataset.abrir;
    abrir({ tela: 'capitulo', livro: Math.floor(k / 1e6), capitulo: Math.floor(k / 1000) % 1000, verso: k % 1000 });
  }));
  c.querySelector('.painel').scrollTop = 0;
}
function ligarPainel(c) {
  $('#flutuante').onclick = () => { guardar('painel', !ler('painel', false)); c.querySelector('.leitor').classList.toggle('com-painel'); $('#flutuante').innerHTML = icone(ler('painel', false) ? 'fechar_painel' : 'painel'); };
  c.querySelector('.painel').addEventListener('click', async (ev) => {
    const a = ev.target.closest('a');
    if (a) await seguirLink(c, a, ev);
  });
}
async function seguirLink(c, a, ev) {
  ev.preventDefault();
  if (a.dataset.biblia) {
    try { mostrarNoTopo(c, t('Texto bíblico'), blocoTexto(await chamar('texto_biblico', { link: a.dataset.biblia }))); }
    catch (erro) { aviso(String(erro)); }
  } else if (a.dataset.meps) {
    const achado = await chamar('achar_meps', { meps: +a.dataset.meps });
    if (achado) abrir({ tela: 'documento', pasta: achado[0], id: achado[1] });
    else aviso(t('Essa publicação não está na sua biblioteca.'));
  } else if (a.dataset.fora) {
    aviso(t('Link externo: {x}', { x: a.dataset.fora }), 5000);
  }
}
function irAoVerso(c, e, verso, realcar) {
  const alvo = c.querySelector(`.documento [id^="v${e.livro}-${e.capitulo}-${verso}-"]`);
  if (!alvo) return;
  alvo.scrollIntoView({ behavior: realcar ? 'auto' : 'smooth', block: 'center' });
  if (realcar) { alvo.classList.add('realce-v'); setTimeout(() => alvo.classList.remove('realce-v'), 2500); }
}
function textoDoVerso(c, e, verso) {
  return [...c.querySelectorAll(`.documento [id^="v${e.livro}-${e.capitulo}-${verso}-"]`)].map((x) => {
    const k = x.cloneNode(true);
    k.querySelectorAll('.vl, .cl, .m, .fn, .tt').forEach((y) => y.remove());
    return k.textContent;
  }).join(' ').replace(/\s+/g, ' ').trim();
}
function menuDoVersiculo(c, e, d, verso, ancora) {
  const partes = c.querySelectorAll(`.documento [id^="v${e.livro}-${e.capitulo}-${verso}-"]`);
  c.querySelector('.documento').classList.add('esmaecido');
  partes.forEach((x) => x.classList.add('escolhido'));
  const limpar = () => { c.querySelector('.documento')?.classList.remove('esmaecido'); partes.forEach((x) => x.classList.remove('escolhido')); };
  const m = menu(ancora, `<div class="barra-verso">
    <button data-o="notas" title="${t('Notas de estudo')}">${icone('painel')}<span>${t('Estudo')}</span></button>
    <button data-o="nota" title="${t('Adicionar uma nota')}">${icone('nota')}<span>${t('Nota')}</span></button>
    <button data-o="copiar" title="${t('Copiar')}">${icone('copiar')}<span>${t('Copiar')}</span></button>
    </div>`);
  m.classList.add('menu-verso');
  const r = ancora.getBoundingClientRect();
  m.style.top = Math.max(90, r.top - m.offsetHeight - 8) + 'px';
  m.style.left = Math.max(56, r.left - 20) + 'px';
  const fora = (ev) => { if (!m.contains(ev.target)) { limpar(); document.removeEventListener('click', fora, true); } };
  setTimeout(() => document.addEventListener('click', fora, true), 0);
  m.onclick = async (ev) => {
    const o = ev.target.closest('[data-o]')?.dataset.o;
    if (o === 'nota') {
      await Marcas.novaNota(verso, null);
    } else if (o === 'copiar') {
      const texto = `${d.titulo}:${verso} ${textoDoVerso(c, e, verso)}`;
      try { await navigator.clipboard.writeText(texto); aviso(t('Versículo copiado.')); } catch (_) { aviso(t('Não deu para copiar.')); }
    } else if (o === 'notas') {
      abrirPainel(c);
      const alvo = c.querySelector(`.painel [data-ir="${verso}"]`);
      if (alvo) alvo.scrollIntoView({ behavior: 'smooth', block: 'start' });
      else aviso(t('Este versículo não tem notas de estudo.'));
    }
    m.remove();
    limpar();
  };
}
function menuDoLeitor(b) {
  const m = menu(b, `<button data-o="texto">${t('Configurações de texto')}</button><button data-o="tela">${t('Tela inteira')}</button>`);
  m.onclick = async (x) => {
    const o = x.target.dataset.o;
    if (o === 'texto') {
      const t = menu(b, `<div class="deslizante"><small>A</small><input type="range" min="13" max="30" value="${ler('tamanho', 18)}"><big>A</big></div>`);
      t.querySelector('input').oninput = (y) => { guardar('tamanho', +y.target.value); $('.documento').style.setProperty('--tam', y.target.value + 'px'); };
    }
    if (o === 'tela') { fecharMenu(); const j = tauri.window.getCurrentWindow(); j.setFullscreen(!(await j.isFullscreen())); }
  };
}
// Rola até o primeiro parágrafo que tem as palavras buscadas e pisca nele.
function irAoTermo(c, termos) {
  const tirar = (x) => x.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
  const palavras = tirar(termos).split(/[^\p{L}\p{N}]+/u).filter(Boolean);
  if (!palavras.length) return;
  const achado = [...c.querySelectorAll('.documento [data-pid]')].find((p) => { const x = tirar(p.textContent); return palavras.every((w) => x.includes(w)); });
  if (!achado) return;
  achado.scrollIntoView({ block: 'center' });
  achado.classList.add('realce-v');
  setTimeout(() => achado.classList.remove('realce-v'), 2500);
}

function guardarHistorico(item) {
  const igual = (x) => x.tela === item.tela && x.pasta === item.pasta && x.id === item.id && x.livro === item.livro && x.capitulo === item.capitulo;
  const h = ler('historico', []).filter((x) => !igual(x));
  h.unshift(item);
  guardar('historico', h.slice(0, 30));
}
const abrirDoHistorico = (x) => abrir(x.tela === 'capitulo' ? { tela: 'capitulo', livro: x.livro, capitulo: x.capitulo } : { tela: 'documento', pasta: x.pasta, id: x.id });

async function backup() {
  const r = await chamar('resumo_pessoal');
  const conta = (x) => t('{g} grifos, {n} notas, {e} etiquetas e {m} marcadores', { g: x.grifos, n: x.notas, e: x.etiquetas, m: x.marcadores });
  const corpo = `<p>${t('Neste computador: {x}.', { x: conta(r) })}</p>
    <p>${t('O backup usa o mesmo formato do JW Library (.jwlibrary): dá para levar para o celular ou trazer de lá.')}</p>`;
  const f = dialogo(t('Backup e restauração'), corpo, [[t('Fechar'), null]]);
  const rodape = f.querySelector('.rodape');
  const criar = document.createElement('button');
  criar.className = 'botao';
  criar.textContent = t('Criar backup');
  const restaurar = document.createElement('button');
  restaurar.className = 'botao leve';
  restaurar.textContent = t('Restaurar backup');
  rodape.prepend(criar, restaurar);
  criar.onclick = async () => {
    const hoje = new Date().toISOString().slice(0, 10);
    const destino = await tauri.dialog.save({ title: t('Criar backup'), defaultPath: `UserdataBackup_${hoje}_JW_Library_on_Linux.jwlibrary`, filters: [{ name: t('Backup do JW Library'), extensions: ['jwlibrary'] }] });
    if (!destino) return;
    try { const x = await chamar('criar_backup', { destino }); f.remove(); aviso(t('Backup criado com {x}.', { x: conta(x) }), 6000); }
    catch (erro) { aviso(t('Não deu para criar o backup: {x}', { x: erro }), 8000); }
  };
  restaurar.onclick = async () => {
    const arquivo = await tauri.dialog.open({ title: t('Restaurar backup'), multiple: false, filters: [{ name: t('Backup do JW Library'), extensions: ['jwlibrary'] }] });
    if (!arquivo) return;
    let x;
    try { x = await chamar('conferir_backup', { arquivo }); } catch (erro) { return aviso(t('Este arquivo não serve: {x}', { x: erro }), 8000); }
    f.remove();
    const quando = x.data ? new Date(x.data).toLocaleString(Idioma.local) : '';
    dialogo(t('Restaurar backup'), `<p>${x.aparelho ? t('O backup de {a} ({d}) tem {x}.', { a: `<b>${esc(x.aparelho)}</b>`, d: esc(quando), x: conta(x) }) : t('O backup tem {x}.', { x: conta(x) })}</p>
      <p>${t('Ele <b>substitui</b> todo o estudo pessoal deste computador. Uma cópia do que existe agora fica guardada.')}</p>`,
      [[t('Cancelar'), null], [t('Restaurar'), async () => {
        try { const y = await chamar('restaurar_backup'); aviso(t('Restaurado: {x}.', { x: conta(y) }), 6000); desenhar(); }
        catch (erro) { aviso(t('Não deu para restaurar: {x}', { x: erro }), 8000); }
      }]]);
  };
}

// ---------- Telas ----------
const TELAS = {
  vazia(c) {
    c.innerHTML = `<div class="vazio">${t('Nada aqui.')}</div>`;
  },

  inicio(c) {
    barra(t('Início'), Idioma.nome, [['buscar', t('Buscar nas publicações'), () => abrir({ tela: 'busca', onde: 'publicacoes' })], ['importar', t('Importar publicações'), escolherArquivos], ['historico', t('Histórico'), historico], ['config', t('Configurações'), configuracoes]]);
    const lidos = ler('historico', []).slice(0, 8);
    const recentes = biblioteca.cartoes.slice().sort((a, b) => b.ano - a.ano || b.edicao - a.edicao).slice(0, 6);
    c.innerHTML = `<div class="tela">
      <div class="boas-vindas"><h1>${t('Bem-vindo ao JW Library on Linux')}</h1>
        <p>${biblioteca.cartoes.length ? t('{n} publicações na sua biblioteca.', { n: biblioteca.cartoes.length }) : t('Importe os arquivos .jwpub que você já tem para começar a ler.')}</p>
        <button class="botao" id="imp">${t('Importar publicações')}</button></div>
      ${lidos.length ? `<div class="secao-titulo">${t('Continuar lendo')}</div><div class="lista">${lidos.map((h, i) =>
        `<button class="item" data-h="${i}"><div><div class="t">${esc(h.titulo)}</div><div class="m">${esc(h.publicacao)}</div></div></button>`).join('')}</div>` : ''}
      ${recentes.length ? `<div class="secao-titulo">${t('Mais recentes na biblioteca')}</div><div class="grade">${recentes.map((r) => cartao(r)).join('')}</div>` : ''}
    </div>`;
    $('#imp').onclick = escolherArquivos;
    c.querySelectorAll('[data-h]').forEach((b) => (b.onclick = () => {
      abrirDoHistorico(lidos[+b.dataset.h]);
    }));
    ligarCartoes(c);
    faixaNovos(c);
    faixaBaixador(c);
  },

  async biblia(c) {
    let dados;
    try { dados = await chamar('livros'); } catch (_) {
      barra(t('Bíblia'), Idioma.nome);
      c.innerHTML = `<div class="vazio"><h2>${t('Nenhuma Bíblia instalada')}</h2><p>${t('Importe o arquivo .jwpub de uma Bíblia (por exemplo, nwtsty_T.jwpub, a Bíblia de Estudo).')}</p><button class="botao" id="imp">${t('Importar')}</button></div>`;
      $('#imp').onclick = escolherArquivos;
      return;
    }
    barra(dados.publicacao, Idioma.nome, [['buscar', t('Buscar na Bíblia'), () => abrir({ tela: 'busca' })], ['historico', t('Histórico'), historico]]);
    const azulejo = (l) => `<button class="livro" data-livro="${l.numero}" data-caps="${l.capitulos}">${esc(l.nome)}</button>`;
    c.innerHTML = `<div class="tela"><div class="testamentos">
      <section><div class="grupo">${t('Escrituras Hebraico-Aramaicas')}</div><div class="livros hebr">${dados.livros.slice(0, 39).map(azulejo).join('')}</div></section>
      <section><div class="grupo">${t('Escrituras Gregas Cristãs')}</div><div class="livros greg">${dados.livros.slice(39).map(azulejo).join('')}</div></section>
    </div></div>`;
    c.querySelectorAll('[data-livro]').forEach((b) => (b.onclick = () => {
      const livro = +b.dataset.livro;
      abrir(+b.dataset.caps === 1 ? { tela: 'capitulo', livro, capitulo: 1 } : { tela: 'livro', livro });
    }));
  },

  async livro(c, e) {
    const dados = await chamar('livros');
    const l = dados.livros[e.livro - 1];
    barra(l.nome, dados.publicacao, [['buscar', t('Buscar na Bíblia'), () => abrir({ tela: 'busca' })]]);
    const nums = Array.from({ length: l.capitulos }, (_, i) => i + 1);
    c.innerHTML = `<div class="tela"><div class="faixa-livro">${esc(l.nome)}</div>
      <div class="capitulos">${nums.map((n) => `<button data-cap="${n}">${n}</button>`).join('')}</div></div>
      ${e.livro > 1 ? `<button class="seta esq" title="${t('Livro anterior')}">${icone('esq')}</button>` : ''}
      ${e.livro < 66 ? `<button class="seta dir" title="${t('Livro seguinte')}">${icone('dir')}</button>` : ''}`;
    c.querySelectorAll('[data-cap]').forEach((b) => (b.onclick = () => abrir({ tela: 'capitulo', livro: e.livro, capitulo: +b.dataset.cap })));
    const trocar = (livro) => { pilhas[secao][pilhas[secao].length - 1] = { tela: 'livro', livro }; desenhar(); };
    c.querySelector('.seta.esq')?.addEventListener('click', () => trocar(e.livro - 1));
    c.querySelector('.seta.dir')?.addEventListener('click', () => trocar(e.livro + 1));
  },

  async capitulo(c, e) {
    const d = await chamar('capitulo', { livro: e.livro, capitulo: e.capitulo });
    const tam = ler('tamanho', 18);
    const comPainel = ler('painel', false);
    barra(d.titulo, d.publicacao, [
      ['buscar', t('Buscar na Bíblia'), () => abrir({ tela: 'busca' })],
      ['marcador', t('Marcadores'), () => Marcas.marcadores()],
      ['historico', t('Histórico'), historico],
      ['mais', t('Mais opções'), (ev, b) => menuDoLeitor(b)],
    ]);
    c.innerHTML = `<div class="leitor${comPainel ? ' com-painel' : ''}">
      <div class="coluna"><article class="documento serifa biblia" style="--tam:${tam}px">${d.html}</article></div>
      <aside class="painel"><h3>${t('Painel de estudo')}</h3><div class="painel-topo"></div><div class="painel-corpo"></div></aside>
    </div>
    ${d.anterior ? `<button class="seta esq" title="${t('Capítulo anterior')}">${icone('esq')}</button>` : ''}
    ${d.seguinte ? `<button class="seta dir" title="${t('Capítulo seguinte')}">${icone('dir')}</button>` : ''}
    <button id="flutuante" title="${t(comPainel ? 'Esconder painel' : 'Mostrar painel')}">${icone(comPainel ? 'fechar_painel' : 'painel')}</button>`;
    const doc = c.querySelector('.documento');
    // Notas de rodapé do capítulo vão do fim do texto para o painel.
    const rodape = doc.querySelector('.groupFootnote');
    const blocos = [];
    for (const n of d.notas) blocos.push({ verso: n.verso, html: `<div class="bloco" data-verso="${n.verso}"><div class="rotulo">${t('Nota de estudo')}</div>${n.html}</div>` });
    rodape?.querySelectorAll('.fn-ref').forEach((f) => {
      const id = f.dataset.fnid;
      const chamada = doc.querySelector(`a.fn[data-fnid="${id}"]`);
      const verso = +(chamada?.closest('.v')?.id.split('-')[2] || 0);
      blocos.push({ verso, html: `<div class="bloco" data-nota="${id}"><div class="rotulo">${t('Nota de rodapé')}</div>${f.innerHTML}</div>` });
    });
    rodape?.remove();
    blocos.sort((a, b) => a.verso - b.verso);
    let ultimo = -1;
    c.querySelector('.painel-corpo').innerHTML = blocos.length ? blocos.map((b) => {
      const cab = b.verso !== ultimo ? `<div class="paragrafo" data-ir="${b.verso}">${e.capitulo}:${b.verso}</div>` : '';
      ultimo = b.verso;
      return cab + b.html;
    }).join('') : `<p class="vazio" style="padding:20px 0">${t('Nenhum conteúdo de estudo.')}</p>`;
    c.querySelectorAll('.painel [data-ir]').forEach((x) => (x.onclick = () => irAoVerso(c, e, +x.dataset.ir)));
    ligarPainel(c);
    const trocar = (par) => { pilhas[secao][pilhas[secao].length - 1] = { tela: 'capitulo', livro: par[0], capitulo: par[1] }; desenhar(); };
    c.querySelector('.seta.esq')?.addEventListener('click', () => trocar(d.anterior));
    c.querySelector('.seta.dir')?.addEventListener('click', () => trocar(d.seguinte));
    doc.addEventListener('click', async (ev) => {
      const letra = ev.target.closest('.m[data-mid]');
      const numero = ev.target.closest('.vl, .cl');
      const a = ev.target.closest('a');
      if (letra) {
        const refs = await chamar('referencias', { livro: e.livro, capitulo: e.capitulo, mid: +letra.dataset.mid });
        mostrarNoTopo(c, t('Referências marginais'), refs.map(blocoTexto).join('') || `<p>${t('Sem referências.')}</p>`);
      } else if (numero) {
        const v = numero.closest('.v');
        if (v) menuDoVersiculo(c, e, d, +v.id.split('-')[2], numero);
      } else if (a?.dataset.fnid) {
        ev.preventDefault();
        mostrarNota(c, a.dataset.fnid);
      } else if (a) {
        await seguirLink(c, a, ev);
      }
    });
    if (e.verso) setTimeout(() => irAoVerso(c, e, e.verso, true), 50);
    guardarHistorico({ tela: 'capitulo', livro: e.livro, capitulo: e.capitulo, titulo: d.titulo, publicacao: d.publicacao });
    await Marcas.montar(c, { tipo: 'biblia', livro: e.livro, capitulo: e.capitulo });
  },

  busca(c, e) {
    const onde = e.onde || 'biblia';
    barra(t(onde === 'biblia' ? 'Buscar na Bíblia' : 'Buscar nas publicações'), Idioma.nome);
    c.innerHTML = `<div class="tela"><div class="abas">${[['biblia', 'BÍBLIA'], ['publicacoes', 'PUBLICAÇÕES']].map(([k, n]) =>
      `<button data-onde="${k}" class="${k === onde ? 'ativa' : ''}">${t(n)}</button>`).join('')}</div>
      <form class="campo-busca"><input type="search" placeholder="${t('Palavras ou expressão')}" value="${esc(e.termos || '')}" autofocus>
      <label><input type="checkbox"${e.exata ? ' checked' : ''}> ${t('Achar a expressão exata')}</label><button class="botao">${t('Buscar')}</button></form>
      <div class="resultado"></div></div>`;
    c.querySelectorAll('[data-onde]').forEach((x) => (x.onclick = () => {
      e.onde = x.dataset.onde;
      e.termos = c.querySelector('input[type=search]').value.trim();
      desenhar();
    }));
    const form = c.querySelector('form');
    const campo = form.querySelector('input[type=search]');
    const exata = form.querySelector('input[type=checkbox]');
    const fazer = async () => {
      e.termos = campo.value.trim();
      e.exata = exata.checked;
      const r = c.querySelector('.resultado');
      if (!e.termos) { r.innerHTML = ''; return; }
      r.innerHTML = `<p class="vazio">${t('Buscando… (a primeira busca prepara o índice e demora alguns segundos)')}</p>`;
      try {
        const b = await chamar(onde === 'biblia' ? 'buscar_biblia' : 'buscar_publicacoes', { termos: e.termos, exata: e.exata });
        const contador = `<div class="contador">${t(b.total === 1 ? '{n} ocorrência' : '{n} ocorrências', { n: b.total })}${b.total > b.achados.length ? t(' (mostrando {n})', { n: b.achados.length }) : ''}</div>`;
        if (onde === 'biblia') {
          r.innerHTML = contador + `<div class="lista">${b.achados.map((x) => `<button class="item achado" data-codigo="${x.codigo}"><div><div class="t">${esc(x.rotulo)}</div><div class="m">${x.trecho}</div></div></button>`).join('')}</div>`;
          r.querySelectorAll('[data-codigo]').forEach((x) => (x.onclick = () => {
            const k = +x.dataset.codigo;
            abrir({ tela: 'capitulo', livro: Math.floor(k / 1e6), capitulo: Math.floor(k / 1000) % 1000, verso: k % 1000 });
          }));
        } else {
          r.innerHTML = contador + `<div class="lista">${b.achados.map((x, i) => `<button class="item achado" data-i="${i}"><div><div class="t">${esc(x.titulo)}</div><div class="m"><small>${esc(x.publicacao)}</small><br>${x.trecho}</div></div></button>`).join('')}</div>`;
          r.querySelectorAll('[data-i]').forEach((x) => (x.onclick = () => {
            const a = b.achados[+x.dataset.i];
            abrir({ tela: 'documento', pasta: a.pasta, id: a.id, procurar: e.termos });
          }));
        }
      } catch (erro) { r.innerHTML = `<p class="vazio">${esc(erro)}</p>`; }
    };
    form.onsubmit = (ev) => { ev.preventDefault(); fazer(); };
    exata.onchange = fazer;
    if (e.termos) fazer();
    campo.focus();
  },

  biblioteca(c, e) {
    const aba = e.aba || 'publicacoes';
    const acoes = [['nuvem', t('Baixar publicações'), baixarPublicacoes], ['buscar', t('Buscar nas publicações'), () => abrir({ tela: 'busca', onde: 'publicacoes' })], ['importar', t('Importar publicações'), escolherArquivos], ['historico', t('Histórico'), historico]];
    if (aba === 'baixados') acoes.unshift(['ordenar', t('Ordenar'), (ev, botao) => {
      const m = menu(botao, ['titulo:Título', 'maior:Maior arquivo', 'recente:Mais recentes'].map((o) => {
        const [k, n] = o.split(':');
        return `<button data-o="${k}">${(e.ordem || 'titulo') === k ? '✓ ' : ''}${t(n)}</button>`;
      }).join(''));
      m.onclick = (x) => { const o = x.target.dataset.o; if (o) { e.ordem = o; desenhar(); } };
    }]);
    barra(t('Biblioteca'), Idioma.nome, acoes);
    const abas = `<div class="abas">${[['publicacoes', 'PUBLICAÇÕES'], ['baixados', 'BAIXADOS']].map(([k, n]) =>
      `<button data-aba="${k}" class="${k === aba ? 'ativa' : ''}">${t(n)}</button>`).join('')}</div>`;
    if (!biblioteca.cartoes.length) {
      c.innerHTML = `<div class="tela">${abas}<div class="vazio"><h2>${t('A biblioteca está vazia')}</h2><p>${t('Arraste arquivos .jwpub para esta janela ou use o botão abaixo.')}</p><button class="botao" id="imp">${t('Importar publicações')}</button></div></div>`;
      $('#imp').onclick = escolherArquivos;
    } else if (aba === 'publicacoes') {
      c.innerHTML = `<div class="tela">${abas}<div class="grade">${biblioteca.categorias.map((k) =>
        `<button class="categoria" data-cat="${k.chave}">${icone(ICONE_CATEGORIA[k.chave] || 'livro')}<span>${esc(k.nome)}</span><span class="quantas">${k.quantas}</span></button>`).join('')}</div></div>`;
      c.querySelectorAll('[data-cat]').forEach((b) => (b.onclick = () => abrir({ tela: 'categoria', chave: b.dataset.cat })));
    } else {
      const ordem = e.ordem || 'titulo';
      const lista = biblioteca.cartoes.slice().sort((a, b) =>
        ordem === 'maior' ? b.tamanho - a.tamanho : ordem === 'recente' ? b.ano - a.ano || b.edicao - a.edicao : a.titulo.localeCompare(b.titulo, Idioma.local));
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
    barra(k ? k.nome : t('Biblioteca'), ano && anos.length > 1 ? String(ano) : Idioma.nome, [['historico', t('Histórico'), historico]]);
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
    barra(s.titulo, Idioma.nome, [['mais', t('Mais opções'), (ev, b) => menuDaPublicacao(b, s.pasta)]]);
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
    barra(d.titulo, d.publicacao, [['marcador', t('Marcadores'), () => Marcas.marcadores()], ['historico', t('Histórico'), historico], ['mais', t('Mais opções'), (ev, b) => menuDoLeitor(b)]]);
    c.innerHTML = `<div class="leitor${comPainel ? ' com-painel' : ''}">
      <div class="coluna"><article class="documento" style="--tam:${tam}px">${d.html}</article></div>
      <aside class="painel"><h3>${t('Painel de estudo')}</h3><div class="painel-topo"></div><div class="painel-corpo"><p class="vazio" style="padding:20px 0">${t('Carregando…')}</p></div></aside>
    </div>
    ${d.anterior ? `<button class="seta esq" title="${t('Anterior')}">${icone('esq')}</button>` : ''}
    ${d.seguinte ? `<button class="seta dir" title="${t('Seguinte')}">${icone('dir')}</button>` : ''}
    <button id="flutuante" title="${t(comPainel ? 'Esconder painel' : 'Mostrar painel')}">${icone(comPainel ? 'fechar_painel' : 'painel')}</button>`;
    montarPainel(c, e);
    const trocar = (id) => { pilhas[secao][pilhas[secao].length - 1] = { tela: 'documento', pasta: e.pasta, id }; desenhar(); };
    c.querySelector('.seta.esq')?.addEventListener('click', () => trocar(d.anterior));
    c.querySelector('.seta.dir')?.addEventListener('click', () => trocar(d.seguinte));
    ligarPainel(c);
    c.querySelector('.documento').addEventListener('click', async (ev) => {
      const a = ev.target.closest('a');
      if (!a) return;
      if (a.dataset.fnid) { ev.preventDefault(); mostrarNota(c, a.dataset.fnid); }
      else await seguirLink(c, a, ev);
    });
    guardarHistorico({ tela: 'documento', pasta: e.pasta, id: e.id, titulo: d.titulo, publicacao: d.publicacao });
    await Marcas.montar(c, { tipo: 'doc', pasta: e.pasta, id: e.id });
    if (e.bloco != null) c.querySelector(`.documento [data-pid="${e.bloco}"]`)?.scrollIntoView({ block: 'center' });
    else if (e.procurar) irAoTermo(c, e.procurar);
  },

  async reunioes(c, e) {
    barra(t('Reuniões'), Idioma.nome, [['historico', t('Histórico'), historico]]);
    const hoje = new Date();
    const seg = e.segunda ? new Date(e.segunda) : new Date(hoje.getFullYear(), hoje.getMonth(), hoje.getDate() - ((hoje.getDay() + 6) % 7));
    const dom = new Date(seg.getFullYear(), seg.getMonth(), seg.getDate() + 6);
    const n = (d) => d.getFullYear() * 10000 + (d.getMonth() + 1) * 100 + d.getDate();
    const mes = (d) => d.toLocaleDateString(Idioma.local, { month: 'long' });
    const rotulo = Idioma.atual === 'en'
      ? (seg.getMonth() === dom.getMonth() ? `${mes(dom)} ${seg.getDate()}-${dom.getDate()}` : `${mes(seg)} ${seg.getDate()} – ${mes(dom)} ${dom.getDate()}`)
      : (seg.getMonth() === dom.getMonth() ? `${seg.getDate()}-${dom.getDate()} de ${mes(dom)}` : `${seg.getDate()} de ${mes(seg)} – ${dom.getDate()} de ${mes(dom)}`);
    const atual = n(hoje) >= n(seg) && n(hoje) <= n(dom);
    const s = await chamar('semana', { data: n(seg) });
    const parte = (titulo, x, falta) => `<div class="reuniao"><h3>${titulo}</h3>${x
      ? `<button class="item" data-pasta="${esc(x.pasta)}" data-id="${x.id}">${x.miniatura ? `<img src="${base}/midia/${encodeURIComponent(x.pasta)}/${encodeURIComponent(x.miniatura)}" alt="">` : ''}<div><div class="t">${esc(x.titulo)}</div><div class="m">${esc(x.publicacao)}</div></div></button>`
      : `<p class="falta">${falta}</p>`}</div>`;
    c.innerHTML = `<div class="tela">
      <div class="semana-nav"><button class="seta-s" data-mover="-7" title="${t('Semana anterior')}">${icone('esq')}</button>
        <div class="data">${rotulo}<small>${atual ? t('Esta semana') : seg.getFullYear()}</small></div>
        <button class="seta-s" data-mover="7" title="${t('Semana seguinte')}">${icone('dir')}</button></div>
      ${parte(t('Vida e Ministério'), s.apostila, t('Importe a Apostila Vida e Ministério que cobre esta semana (arquivo mwb_….jwpub).'))}
      ${parte(t('Estudo de A Sentinela'), s.sentinela, t('Importe a edição de estudo de A Sentinela que cobre esta semana (arquivo w_….jwpub).'))}
    </div>`;
    c.querySelectorAll('[data-mover]').forEach((b) => (b.onclick = () => {
      const d = new Date(seg.getFullYear(), seg.getMonth(), seg.getDate() + +b.dataset.mover);
      e.segunda = d.toISOString();
      desenhar();
    }));
    c.querySelectorAll('.item[data-pasta]').forEach((b) => (b.onclick = () => abrir({ tela: 'documento', pasta: b.dataset.pasta, id: +b.dataset.id })));
  },

  async estudo(c, e) {
    barra(t('Notas e etiquetas'), t('Estudo Pessoal'), [
      ['nuvem', t('Backup e restauração'), () => backup()],
      ['historico', t('Histórico'), historico],
    ]);
    const [notas, etiquetas] = await Promise.all([chamar('notas', { filtro: e.filtro || '', etiqueta: e.etiqueta || null }), chamar('etiquetas')]);
    const data = (t) => { try { return new Date(t).toLocaleDateString(Idioma.local); } catch (_) { return ''; } };
    c.innerHTML = `<div class="tela">
      <form class="campo-busca"><input type="search" placeholder="${t('Buscar nas notas')}" value="${esc(e.filtro || '')}"></form>
      <div class="etiquetas">${etiquetas.length ? etiquetas.map(([nome, n]) => `<span class="chip${e.etiqueta === nome ? ' ativa' : ''}" data-etiqueta="${esc(nome)}">${esc(nome)} <small>${n}</small><button data-tirar="${esc(nome)}" title="${t('Apagar a etiqueta')}">×</button></span>`).join('') : `<span class="dica">${t('Dica: escreva etiquetas nas notas (separadas por vírgula) para agrupar por assunto.')}</span>`}</div>
      ${notas.length ? `<div class="mural">${notas.map((n, i) => `<button class="nota-mural cor-${n.cor || 0}" data-i="${i}">
          <div class="quando">${data(n.alterada)}</div>
          ${n.titulo ? `<div class="t">${esc(n.titulo)}</div>` : ''}
          <div class="corpo">${esc(n.texto).replace(/\n/g, '<br>') || `<i>${t('(sem texto)')}</i>`}</div>
          ${n.etiquetas.length ? `<div class="mini-etiquetas">${n.etiquetas.map(esc).join(' · ')}</div>` : ''}
          <div class="origem">${esc(n.origem || n.simbolo)}</div></button>`).join('')}</div>`
        : `<div class="vazio"><h2>${t(e.filtro || e.etiqueta ? 'Nenhuma nota encontrada' : 'Nenhuma nota ainda')}</h2><p>${t('Selecione um trecho no leitor e escolha "Adicionar uma nota", ou clique no número de um versículo.')}</p></div>`}
    </div>`;
    const form = c.querySelector('form');
    form.onsubmit = (x) => { x.preventDefault(); e.filtro = form.querySelector('input').value; desenhar(); };
    form.querySelector('input').addEventListener('search', () => { e.filtro = form.querySelector('input').value; desenhar(); });
    c.querySelectorAll('[data-etiqueta]').forEach((x) => (x.onclick = (ev) => {
      if (ev.target.dataset.tirar) return;
      e.etiqueta = e.etiqueta === x.dataset.etiqueta ? null : x.dataset.etiqueta;
      desenhar();
    }));
    c.querySelectorAll('[data-tirar]').forEach((x) => (x.onclick = () => {
      dialogo(t('Apagar etiqueta'), `<p>${t('A etiqueta "{x}" sai de todas as notas. As notas continuam.', { x: esc(x.dataset.tirar) })}</p>`, [[t('Cancelar'), null], [t('Apagar'), async () => {
        await chamar('apagar_etiqueta', { nome: x.dataset.tirar });
        if (e.etiqueta === x.dataset.tirar) e.etiqueta = null;
        desenhar();
      }]]);
    }));
    c.querySelectorAll('.nota-mural').forEach((x) => (x.onclick = async () => {
      const n = notas[+x.dataset.i];
      if (n.livro) return abrir({ tela: 'capitulo', livro: n.livro, capitulo: n.capitulo, verso: n.bloco });
      const achado = n.meps ? await chamar('achar_meps', { meps: n.meps }) : null;
      if (achado) abrir({ tela: 'documento', pasta: achado[0], id: achado[1], bloco: n.bloco });
      else aviso(t('A publicação desta nota não está na sua biblioteca.'));
    }));
  },
};

// Troca o idioma: o núcleo passa a dar categorias e livros da Bíblia no idioma novo
// e a janela é redesenhada do zero.
async function trocarIdioma(novo) {
  Idioma.definir(novo);
  try { await chamar('definir_idioma', { idioma: novo }); } catch (_) {}
  location.reload();
}

// Primeira abertura: escolher o idioma antes de tudo.
function escolherIdioma() {
  return new Promise((pronto) => {
    const f = document.createElement('div');
    f.className = 'escolha-idioma';
    f.innerHTML = `<div class="caixa"><img src="icone.svg" alt="" width="96" height="96">
      <h1>JW Library on Linux</h1>
      <p>Escolha o idioma do aplicativo<br><span>Choose the app language</span></p>
      <div class="opcoes">${Object.entries(IDIOMAS).map(([k, x]) => `<button class="botao" data-idioma="${k}">${x.nome}</button>`).join('')}</div>
      <small>Dá para trocar depois em Configurações. · You can change it later in Settings.</small></div>`;
    document.body.appendChild(f);
    f.querySelectorAll('[data-idioma]').forEach((b) => (b.onclick = () => { Idioma.definir(b.dataset.idioma); f.remove(); pronto(); }));
  });
}

(async () => {
  aplicarTema();
  if (!Idioma.escolhido) await escolherIdioma();
  document.documentElement.lang = Idioma.local;
  try { await chamar('definir_idioma', { idioma: Idioma.atual }); } catch (_) {}
  document.querySelector('#voltar').title = t('Voltar');
  document.querySelector('#soltar').textContent = t('Solte os arquivos .jwpub para importar');
  montarTrilho();
  try { await carregarBiblioteca(); } catch (e) { aviso(t('Erro ao ler a biblioteca: {x}', { x: e })); }
  ouvirArrastar();
  try { temBaixador = await chamar('tem_baixador'); } catch (_) {}
  // Ao voltar para a janela (por exemplo, depois de usar o baixador), relê a
  // biblioteca e redesenha as telas que mostram a lista de publicações.
  window.addEventListener('focus', async () => {
    const antes = biblioteca.cartoes.map((x) => x.pasta + x.tamanho).join();
    try { await carregarBiblioteca(); } catch (_) { return; }
    if (biblioteca.cartoes.map((x) => x.pasta + x.tamanho).join() === antes) return;
    const p = pilhas[secao];
    if (['inicio', 'biblioteca', 'categoria', 'reunioes', 'biblia'].includes(p[p.length - 1].tela)) desenhar();
  });
  desenhar();
})();
