// A Biblioteca como a do JW Library: todas as categorias do catálogo do jw.org
// (mesmo sem nada instalado), e em cada uma as publicações instaladas e as que
// dá para baixar (nuvem + tamanho). Comandos em app/src/baixar.rs (catalogo_*).
const BibliotecaJW = (() => {
  const IMAGENS = 'https://app.jw-cdn.org/catalogs/publications/';
  const PERIODICOS = ['sentinela', 'despertai', 'apostilas', 'ministerio'];
  const LINGUAS = [['T', 'Português (Brasil)'], ['TPO', 'Português (Portugal)'], ['E', 'English'], ['S', 'Español'], ['F', 'Français'], ['X', 'Deutsch'], ['I', 'Italiano']];
  let info = null; // {versao, baixado_em} do catálogo local
  let atualizando = null; // promessa do download do catálogo
  const baixando = {}; // chave → {feito, total}

  function lingua() {
    try { return JSON.parse(localStorage.getItem('lingua_biblioteca')) || (Idioma.atual === 'en' ? 'E' : 'T'); } catch (_) { return 'T'; }
  }
  function nomeLingua() {
    const fixa = LINGUAS.find(([k]) => k === lingua());
    if (fixa) return fixa[1];
    try { return JSON.parse(localStorage.getItem('lingua_biblioteca_nome')) || lingua(); } catch (_) { return lingua(); }
  }
  // Todas as línguas do jw.org, com busca (como o seletor de idiomas do JW Library).
  async function escolherLingua() {
    const f = dialogo(t('Idioma das publicações'), `<input type="search" class="busca-lingua" placeholder="${t('Buscar idioma')}"><div class="lista-linguas"><p class="vazio">${t('Carregando…')}</p></div>`);
    const caixa = f.querySelector('.lista-linguas');
    let todas = [];
    try { todas = await chamar('linguas_jw'); } catch (erro) { todas = LINGUAS.map(([code, vernacular]) => ({ code, vernacular, name: vernacular })); }
    const tirar = (x) => (x || '').normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
    const mostrar = (filtro) => {
      const f2 = tirar(filtro);
      const lista = todas.filter((l) => !f2 || tirar(l.vernacular).includes(f2) || tirar(l.name).includes(f2) || tirar(l.code) === f2);
      caixa.innerHTML = lista.slice(0, 300).map((l) => `<button class="item-lingua${l.code === lingua() ? ' atual' : ''}" data-l="${esc(l.code)}" data-n="${esc(l.vernacular)}">
        <b>${esc(l.vernacular)}</b><span>${esc(l.name)}${l.isSignLanguage ? ` · ${t('língua de sinais')}` : ''}</span></button>`).join('') || `<p class="vazio">${t('Nada encontrado.')}</p>`;
    };
    mostrar('');
    const busca = f.querySelector('.busca-lingua');
    busca.oninput = () => mostrar(busca.value);
    busca.focus();
    caixa.onclick = async (ev) => {
      const b = ev.target.closest('[data-l]');
      if (!b) return;
      const codigo = b.dataset.l;
      caixa.innerHTML = `<p class="vazio">${t('Preparando o idioma {x}… (só na primeira vez)', { x: esc(b.dataset.n) })}</p>`;
      try {
        await chamar('catalogo_adicionar_lingua', { lingua: codigo });
        try { localStorage.setItem('lingua_biblioteca', JSON.stringify(codigo)); localStorage.setItem('lingua_biblioteca_nome', JSON.stringify(b.dataset.n)); } catch (_) {}
        f.remove();
        await carregarInfo();
        desenhar();
      } catch (erro) { caixa.innerHTML = `<p class="vazio">${esc(erro)}</p>`; }
    };
  }

  async function carregarInfo() {
    try { info = await chamar('catalogo_info'); } catch (_) { info = null; }
    return info;
  }
  // Baixa o catálogo na primeira vez e depois a cada 7 dias (em segundo plano).
  async function garantir(forcar = false) {
    if (atualizando) return atualizando;
    await carregarInfo();
    const velho = !info || Date.now() / 1000 - info.baixado_em > 7 * 86400;
    if (!forcar && !velho) return false;
    atualizando = chamar('catalogo_atualizar')
      .then(async (mudou) => { await carregarInfo(); return mudou; })
      .catch((erro) => { if (forcar || !info) aviso(t('Não deu para baixar o catálogo: {x}', { x: erro }), 8000); return false; })
      .finally(() => { atualizando = null; });
    return atualizando;
  }

  const capaUrl = (x) => (x.pasta && x.capa_local ? `${base}/midia/${encodeURIComponent(x.pasta)}/${encodeURIComponent(x.capa_local)}` : x.capa ? IMAGENS + x.capa : '');

  // ---------- Categorias ----------
  async function publicacoes(c, abas) {
    const lugar = c.querySelector('.lugar-publicacoes');
    if (!info) {
      lugar.innerHTML = `<div class="faixa-catalogo">${icone('nuvem')}<div class="texto"><b>${t('Baixando o catálogo de publicações do jw.org…')}</b><span id="pct-catalogo">${t('Uma vez só; depois é atualizado sozinho toda semana.')}</span></div></div>
        <div class="grade">${biblioteca.categorias.map((k) => `<button class="categoria" data-cat="${k.chave}">${icone(ICONE_CATEGORIA[k.chave] || 'livro')}<span>${esc(k.nome)}</span><span class="quantas">${k.quantas}</span></button>`).join('')}</div>`;
      ligarCategorias(lugar);
      garantir().then(() => { if (info && document.body.contains(lugar)) desenhar(); });
      return;
    }
    const cats = await chamar('catalogo_categorias', { lingua: lingua() });
    if (!cats.length) { lugar.innerHTML = `<div class="vazio"><h2>${t('Nenhuma publicação para ler neste idioma')}</h2><p>${t('O catálogo do jw.org não tem publicações .jwpub neste idioma. Veja as abas VÍDEO e ÁUDIO.')}</p></div>`; return; }
    lugar.innerHTML = `<div class="grade">${cats.map((k) => `<button class="categoria" data-cat="${k.chave}">${icone(ICONE_CATEGORIA[k.chave] || 'livro')}<span>${esc(k.nome)}</span><span class="quantas" title="${t('{i} na sua biblioteca, {n} no total', { i: k.instaladas, n: k.quantas })}">${k.instaladas ? `${k.instaladas} / ` : ''}${k.quantas}</span></button>`).join('')}</div>`;
    ligarCategorias(lugar);
    garantir(); // confere se há catálogo novo (no máximo 1x por semana)
  }
  function ligarCategorias(lugar) {
    lugar.querySelectorAll('[data-cat]').forEach((b) => (b.onclick = () => abrir({ tela: 'categoria', chave: b.dataset.cat })));
  }

  // ---------- Uma categoria ----------
  async function categoria(c, e) {
    const nomeCat = (biblioteca.categorias.find((x) => x.chave === e.chave) || {}).nome;
    const acoes = [['idioma', t('Idioma das publicações'), (ev, b) => escolherLingua(b)], ['historico', t('Histórico'), historico]];
    if (PERIODICOS.includes(e.chave) && e.ano == null) {
      const anos = await chamar('catalogo_anos', { lingua: lingua(), categoria: e.chave });
      barra(nomeCat || t('Biblioteca'), nomeLingua(), acoes);
      c.innerHTML = `<div class="tela"><div class="grade">${anos.map((a) => `<button class="categoria ano" data-ano="${a}">${a}</button>`).join('')}</div></div>`;
      c.querySelectorAll('[data-ano]').forEach((b) => (b.onclick = () => abrir({ tela: 'categoria', chave: e.chave, ano: +b.dataset.ano })));
      return;
    }
    barra(nomeCat || t('Biblioteca'), e.ano ? `${e.ano} · ${nomeLingua()}` : nomeLingua(), acoes);
    const lista = await chamar('catalogo_publicacoes', { lingua: lingua(), categoria: e.chave, ano: e.ano ?? null });
    if (!lista.length) { c.innerHTML = `<div class="vazio">${t('Nada aqui.')}</div>`; return; }
    c.innerHTML = `<div class="tela"><div class="grade">${lista.map((x, i) => cartaoJW(x, i)).join('')}</div></div>`;
    c.querySelectorAll('.cartao-jw').forEach((b) => (b.onclick = (ev) => {
      const x = lista[+b.dataset.i];
      const mais = ev.target.closest('[data-mais]');
      if (mais) { ev.stopPropagation(); return menuJW(mais, x, e); }
      if (x.pasta) return abrir({ tela: 'sumario', pasta: x.pasta });
      baixar(x);
    }));
  }

  function cartaoJW(x, i) {
    const img = capaUrl(x);
    const titulo = x.titulo_edicao || x.titulo;
    const prog = baixando[x.chave];
    const pe = prog
      ? `<div class="barrinha"><div style="width:${prog.total ? Math.round((prog.feito * 100) / prog.total) : 2}%"></div></div>`
      : x.pasta
        ? `<div class="m">${x.ano}${x.atualizar ? ` · <span class="atualizavel">${t('versão nova')}</span>` : ''}</div>`
        : `<div class="m nuvem-baixar">${icone('nuvem')} ${megas(x.tamanho)}</div>`;
    return `<button class="cartao cartao-jw${x.pasta ? '' : ' nao-instalada'}" data-i="${i}" data-chave="${esc(x.chave)}">
      ${img ? `<img src="${img}" alt="" loading="lazy" onerror="this.replaceWith(Object.assign(document.createElement('div'),{className:'semcapa'}))">` : '<div class="semcapa"></div>'}
      <div><div class="t">${esc(titulo)}</div>${pe}</div>
      <span class="mais-cartao" role="button" title="${t('Mais opções')}" data-mais="1">${icone('mais')}</span></button>`;
  }

  function menuJW(ancora, x, e) {
    const opcoes = [];
    if (x.pasta) opcoes.push(['abrir', t('Abrir')]);
    else opcoes.push(['baixar', t('Baixar')]);
    if (x.atualizar) opcoes.push(['baixar', t('Atualizar')]);
    if (PERIODICOS.includes(e.chave)) opcoes.push(['novas', t('Baixar as edições novas sozinho')]);
    if (x.pasta) opcoes.push(['excluir', t('Excluir')]);
    const m = menu(ancora, opcoes.map(([k, n]) => `<button data-o="${k}">${n}</button>`).join(''));
    m.onclick = async (ev) => {
      const o = ev.target.closest('[data-o]')?.dataset.o;
      fecharMenu();
      if (o === 'abrir') abrir({ tela: 'sumario', pasta: x.pasta });
      if (o === 'baixar') baixar(x);
      if (o === 'excluir') removerPublicacao(x.pasta, x.titulo_edicao || x.titulo, x.tamanho);
      if (o === 'novas') {
        try {
          await chamar('baixar_escolher', { sigla: x.simbolo, lingua: lingua(), edicao: '', novas: true });
          aviso(t('Pronto: as edições novas serão baixadas sozinhas. Veja em Atualizações automáticas.'), 6000);
        } catch (erro) { aviso(String(erro)); }
      }
    };
  }

  async function baixar(x) {
    baixando[x.chave] = { feito: 0, total: x.tamanho };
    redesenharCartao(x.chave);
    try {
      await chamar('baixar_publicacao', { simbolo: x.simbolo, lingua: lingua(), edicao: x.edicao });
    } catch (erro) {
      delete baixando[x.chave];
      redesenharCartao(x.chave);
      aviso(String(erro), 6000);
    }
  }
  function redesenharCartao(chave) {
    const b = document.querySelector(`.cartao-jw[data-chave="${CSS.escape(chave)}"] .barrinha div`);
    const p = baixando[chave];
    if (b && p) b.style.width = `${p.total ? Math.round((p.feito * 100) / p.total) : 2}%`;
    else if (document.querySelector(`.cartao-jw[data-chave="${CSS.escape(chave)}"]`)) desenhar();
  }

  const ouvir = tauri.event?.listen;
  if (ouvir) {
    ouvir('baixar-progresso', ({ payload }) => {
      if (!baixando[payload.chave]) return;
      baixando[payload.chave] = { feito: payload.feito, total: payload.total };
      redesenharCartao(payload.chave);
    });
    ouvir('baixar-fim', ({ payload }) => {
      const tinha = Object.keys(baixando).length;
      for (const k of Object.keys(baixando)) delete baixando[k];
      if (tinha && payload.desfechos?.some((d) => d.Erro)) aviso(t('Não deu para baixar: {x}', { x: payload.desfechos.find((d) => d.Erro).Erro[1] }), 8000);
      if (tinha) carregarBiblioteca().then(() => { const p = pilhas[secao]; if (['categoria', 'biblioteca'].includes(p[p.length - 1].tela)) desenhar(); });
    });
    ouvir('catalogo-progresso', ({ payload }) => {
      const el = document.getElementById('pct-catalogo');
      const [feito, total] = payload;
      if (el && total) el.textContent = `${Math.round((feito * 100) / total)}% · ${megas(feito)} / ${megas(total)}`;
    });
  }

  return { publicacoes, categoria, garantir, carregarInfo, tem: () => !!info, lingua, nomeLingua, escolherLingua };
})();
