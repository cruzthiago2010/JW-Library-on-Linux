// As abas VÍDEO e ÁUDIO da Biblioteca (como no JW Library): categorias do jw.org,
// assistir/ouvir dentro do app e baixar para ver sem internet.
// Comandos em app/src/midia.rs; os baixados tocam por conteudo://localhost/video/<nome>.
const Midia = (() => {
  const RAIZ = { video: 'VideoOnDemand', audio: 'Audio' };
  let baixadas = [];
  let baseLocal = null; // http://127.0.0.1:<porta>/<chave> (o WebKit não toca vídeo de conteudo://)
  const progresso = {}; // nome → {feito, total}

  const lingua = () => BibliotecaJW.lingua();
  const imagem = (x, larga = true) => {
    const im = x.images || {};
    const ordem = larga ? ['lss', 'wss', 'pnr', 'lsr', 'sqr', 'cvr'] : ['sqr', 'cvr', 'lss', 'wss'];
    for (const k of ordem) { const v = im[k]; if (v && (v.lg || v.md || v.sm)) return v.lg || v.md || v.sm; }
    return '';
  };
  const nomeDoArquivo = (url) => (url || '').split(/[?#]/)[0].split('/').pop();
  const baixada = (url) => baixadas.find((b) => b.nome === nomeDoArquivo(url));
  async function carregarBaixadas() {
    try { baixadas = await chamar('midia_baixadas'); } catch (_) { baixadas = []; }
    if (!baseLocal) { try { baseLocal = await chamar('midia_base'); } catch (_) {} }
  }

  // O melhor arquivo para tocar: mp4 até 720p (vídeo) ou o primeiro mp3 (áudio).
  function arquivoDe(x) {
    const fs = (x.files || []).filter((f) => f.progressiveDownloadURL);
    const mp3 = fs.find((f) => /mp3|audio/.test(f.mimetype || ''));
    if (mp3) return { url: mp3.progressiveDownloadURL, tamanho: mp3.filesize, tipo: 'audio' };
    const mp4 = fs.filter((f) => /mp4/.test(f.mimetype || '')).sort((a, b) => parseInt(b.label) - parseInt(a.label));
    const f = mp4.find((f) => parseInt(f.label) <= 720) || mp4[mp4.length - 1];
    return f ? { url: f.progressiveDownloadURL, tamanho: f.filesize, tipo: 'video', rotulo: f.label } : null;
  }

  // ---------- Raiz da aba ----------
  async function montar(lugar, aba) {
    await carregarBaixadas();
    lugar.innerHTML = `<p class="vazio">${t('Carregando…')}</p>`;
    let cat;
    try { cat = (await chamar('midia_categoria', { lingua: lingua(), chave: RAIZ[aba] })).category; }
    catch (erro) { lugar.innerHTML = `<div class="vazio"><h2>${t('Sem conexão com o jw.org')}</h2><p>${esc(erro)}</p></div>${secaoBaixadas(aba)}`; ligarBaixadas(lugar); return; }
    const subs = cat.subcategories || [];
    const extra = aba === 'audio' ? `<button class="azulejo-midia" data-biblia="1"><div class="img biblia-audio">${icone('biblia')}</div><span>${t('Bíblia em áudio')}</span></button>` : '';
    lugar.innerHTML = `<form class="campo-busca busca-midia"><input type="search" placeholder="${t(aba === 'video' ? 'Buscar vídeos' : 'Buscar áudios')}"><button class="botao">${t('Buscar')}</button></form>
      <div class="resultado-midia"></div>${secaoBaixadas(aba)}<div class="grade-midia">${extra}${subs.map((s) =>
      `<button class="azulejo-midia" data-chave="${esc(s.key)}">${imagem(s) ? `<img src="${esc(imagem(s))}" alt="" loading="lazy">` : `<div class="img">${icone(aba)}</div>`}<span>${esc(s.name)}</span></button>`).join('')}</div>`;
    lugar.querySelectorAll('[data-chave]').forEach((b) => (b.onclick = () => abrir({ tela: 'midia', aba, chave: b.dataset.chave })));
    lugar.querySelector('[data-biblia]')?.addEventListener('click', () => abrir({ tela: 'audiobiblia' }));
    ligarBaixadas(lugar);
    const form = lugar.querySelector('.busca-midia');
    form.onsubmit = async (ev) => {
      ev.preventDefault();
      const termos = form.querySelector('input').value.trim();
      const r = lugar.querySelector('.resultado-midia');
      if (!termos) { r.innerHTML = ''; return; }
      r.innerHTML = `<p class="vazio">${t('Buscando…')}</p>`;
      try {
        const lista = await chamar('midia_buscar', { lingua: lingua(), termos, tipo: aba });
        r.innerHTML = lista.length ? `<div class="secao-titulo">${t(lista.length === 1 ? '{n} ocorrência' : '{n} ocorrências', { n: lista.length })}</div><div class="grade-midia">${lista.map((x) => cartaoMidia(x)).join('')}</div>`
          : `<p class="vazio">${t('Nada encontrado.')}</p>`;
        ligarCartoes(r, lista);
      } catch (erro) { r.innerHTML = `<p class="vazio">${esc(erro)}</p>`; }
    };
  }

  // Os itens do catálogo local não trazem os arquivos: vêm da API na hora.
  async function completar(x) {
    if ((x.files || []).length) return x;
    const chave = x.languageAgnosticNaturalKey || x.guid;
    const novo = await chamar('midia_item', { lingua: lingua(), chave });
    x.files = novo.files || [];
    return x;
  }
  function ligarCartoes(raiz, lista) {
    raiz.querySelectorAll('.cartao-midia[data-guid]').forEach((b) => (b.onclick = async (ev) => {
      const baixar_ = !!ev.target.closest('[data-baixar]');
      let x = lista.find((y) => y.guid === b.dataset.guid);
      const local = baixadas.find((y) => y.titulo === x.title);
      if (local && !baixar_) return tocar({ titulo: local.titulo, imagem: local.imagem, tipo: local.tipo, local: local.nome });
      try { x = await completar(x); } catch (erro) { return aviso(String(erro), 6000); }
      const arq = arquivoDe(x);
      if (!arq) return aviso(t('Este item não tem arquivo para tocar.'));
      if (baixar_) return baixar(x, arq);
      tocar({ titulo: x.title, imagem: imagem(x), tipo: arq.tipo, url: arq.url, local: baixada(arq.url)?.nome, x, arq });
    }));
  }

  function secaoBaixadas(aba) {
    const daqui = baixadas.filter((b) => b.tipo === aba);
    if (!daqui.length) return '';
    return `<div class="secao-titulo">${t('Baixados para ver sem internet')}</div><div class="grade-midia pequenas">${daqui.map((b) =>
      `<button class="cartao-midia" data-baixada="${esc(b.nome)}">${b.imagem ? `<img src="${esc(b.imagem)}" alt="" loading="lazy" onerror="this.style.visibility='hidden'">` : `<div class="img">${icone(aba)}</div>`}
        <div class="t">${esc(b.titulo)}</div><div class="m">${megas(b.tamanho)}</div></button>`).join('')}</div>`;
  }
  function ligarBaixadas(lugar) {
    lugar.querySelectorAll('[data-baixada]').forEach((b) => (b.onclick = () => {
      const x = baixadas.find((y) => y.nome === b.dataset.baixada);
      tocar({ titulo: x.titulo, imagem: x.imagem, tipo: x.tipo, local: x.nome });
    }));
  }

  // ---------- Uma categoria ----------
  async function categoria(c, e) {
    await carregarBaixadas();
    barra(t(e.aba === 'video' ? 'Vídeos' : 'Áudios'), BibliotecaJW.nomeLingua(), [['historico', t('Histórico'), historico]]);
    c.innerHTML = `<div class="tela"><p class="vazio">${t('Carregando…')}</p></div>`;
    const cat = (await chamar('midia_categoria', { lingua: lingua(), chave: e.chave })).category;
    barra(cat.name, BibliotecaJW.nomeLingua(), [['historico', t('Histórico'), historico]]);
    const subs = (cat.subcategories || []).filter((s) => (s.media || []).length || (s.subcategories || []).length);
    const midias = cat.media || [];
    let html = '<div class="tela">';
    if (cat.description) html += `<p class="dica">${esc(cat.description)}</p>`;
    // Subcategorias com vídeos: uma faixa para cada (como no site), com "Ver tudo".
    for (const s of subs) {
      const lista = s.media || [];
      html += `<div class="secao-titulo linha-sub">${esc(s.name)} <button class="link" data-sub="${esc(s.key)}">${t('Ver tudo')}</button></div>
        <div class="grade-midia">${lista.slice(0, 8).map((x) => cartaoMidia(x)).join('')}</div>`;
    }
    if (midias.length) html += `<div class="grade-midia">${midias.map((x) => cartaoMidia(x)).join('')}</div>`;
    if (!subs.length && !midias.length) html += `<div class="vazio">${t('Nada aqui.')}</div>`;
    c.innerHTML = html + '</div>';
    const todos = [...midias, ...subs.flatMap((s) => s.media || [])];
    c.querySelectorAll('[data-sub]').forEach((b) => (b.onclick = () => abrir({ tela: 'midia', aba: e.aba, chave: b.dataset.sub })));
    ligarCartoes(c, todos);
  }

  function cartaoMidia(x) {
    const arq = arquivoDe(x);
    const ja = (arq && baixada(arq.url)) || baixadas.find((b) => b.titulo === x.title);
    const prog = arq ? progresso[nomeDoArquivo(arq.url)] : progresso['t:' + x.title];
    return `<button class="cartao-midia" data-guid="${esc(x.guid)}">
      ${imagem(x) ? `<img src="${esc(imagem(x))}" alt="" loading="lazy">` : '<div class="img"></div>'}
      ${x.durationFormattedMinSec ? `<span class="duracao">${esc(x.durationFormattedMinSec)}</span>` : ''}
      <div class="t">${esc(x.title)}</div>
      ${prog ? `<div class="barrinha"><div style="width:${prog.total ? Math.round(prog.feito * 100 / prog.total) : 2}%"></div></div>`
        : ja ? `<div class="m">✓ ${t('Baixado')}</div>`
        : `<div class="m"><span class="nuvem-baixar" data-baixar="1" title="${t('Baixar para ver sem internet')}">${icone('nuvem')}${arq ? ` ${megas(arq.tamanho || 0)}` : ''}</span></div>`}
    </button>`;
  }

  // ---------- Bíblia em áudio ----------
  async function biblia(c, e) {
    barra(t('Bíblia em áudio'), BibliotecaJW.nomeLingua(), [['historico', t('Histórico'), historico]]);
    if (!e.livro) {
      let nomes = [];
      try { nomes = await chamar('nomes_dos_livros'); } catch (_) { nomes = Array.from({ length: 66 }, (_, i) => String(i + 1)); }
      const botao = (n, i) => `<button class="livro" data-livro="${i + 1}">${esc(n)}</button>`;
      c.innerHTML = `<div class="tela"><div class="testamentos"><section><div class="grupo">${t('Escrituras Hebraico-Aramaicas')}</div><div class="livros hebr">${nomes.slice(0, 39).map(botao).join('')}</div></section><section><div class="grupo">${t('Escrituras Gregas Cristãs')}</div><div class="livros greg">${nomes.slice(39).map((n, i) => botao(n, i + 39)).join('')}</div></section></div></div>`;
      c.querySelectorAll('[data-livro]').forEach((b) => (b.onclick = () => abrir({ tela: 'audiobiblia', livro: +b.dataset.livro, nome: b.textContent })));
      return;
    }
    barra(e.nome || t('Bíblia em áudio'), t('Bíblia em áudio'), [['historico', t('Histórico'), historico]]);
    c.innerHTML = `<div class="tela"><p class="vazio">${t('Carregando…')}</p></div>`;
    await carregarBaixadas();
    const caps = await chamar('midia_biblia', { lingua: lingua(), livro: e.livro });
    c.innerHTML = `<div class="tela"><div class="lista">${caps.map((x, i) => {
      const ja = baixada(x.url);
      return `<div class="item faixa-audio"><button class="tocar" data-i="${i}" title="${t('Ouvir')}">${icone('play')}</button>
        <div class="t">${esc(e.nome || '')} ${x.capitulo}</div><div class="m">${Math.round(x.duracao / 60)} min</div>
        ${ja ? `<span class="m">✓ ${t('Baixado')}</span>` : `<button class="link" data-baixar="${i}">${icone('nuvem')} ${megas(x.tamanho)}</button>`}</div>`;
    }).join('')}</div></div>`;
    c.querySelectorAll('[data-i]').forEach((b) => (b.onclick = () => {
      const x = caps[+b.dataset.i];
      tocar({ titulo: `${e.nome} ${x.capitulo}`, tipo: 'audio', url: x.url, local: baixada(x.url)?.nome });
    }));
    c.querySelectorAll('[data-baixar]').forEach((b) => (b.onclick = () => {
      const x = caps[+b.dataset.baixar];
      baixar({ title: `${e.nome} ${x.capitulo}` }, { url: x.url, tamanho: x.tamanho, tipo: 'audio' });
    }));
  }

  // ---------- Tocar ----------
  function tocar({ titulo, imagem: img, tipo, url, local, x, arq }) {
    document.querySelectorAll('.tocador').forEach((p) => p.remove());
    const fonte = local ? `${baseLocal || base}/video/${encodeURIComponent(local)}` : url;
    const p = document.createElement('div');
    p.className = `tocador ${tipo}`;
    p.innerHTML = `<div class="cab"><b>${esc(titulo)}</b><span class="acoes-tocador">
      ${!local && x && arq ? `<button class="link" data-o="baixar">${icone('nuvem')} ${t('Baixar')}</button>` : ''}
      ${local ? `<button class="link" data-o="apagar">${icone('lixeira')} ${t('Apagar o baixado')}</button>` : ''}
      <button class="link" data-o="fora">${t('Abrir no reprodutor do sistema')}</button>
      <button class="fechar-faixa" data-o="fechar" title="${t('Fechar')}">×</button></span></div>
      ${tipo === 'video' ? `<video controls autoplay src="${esc(fonte)}"${img ? ` poster="${esc(img)}"` : ''}></video>` : `<audio controls autoplay src="${esc(fonte)}"></audio>`}
      <div class="erro-tocar" hidden>${t('O app não conseguiu tocar este arquivo. Use "Abrir no reprodutor do sistema".')}</div>`;
    const el = p.querySelector('video, audio');
    el.onerror = () => { p.querySelector('.erro-tocar').hidden = false; };
    p.onclick = async (ev) => {
      const o = ev.target.closest('[data-o]')?.dataset.o;
      if (o === 'fechar') { el.pause(); p.remove(); }
      if (o === 'baixar') baixar(x, arq);
      if (o === 'fora') { el.pause(); chamar('midia_abrir_fora', local ? { nome: local } : { url }).catch((erro) => aviso(String(erro))); }
      if (o === 'apagar') {
        el.pause();
        try { await chamar('midia_apagar', { nome: local }); aviso(t('Apagado.')); p.remove(); await carregarBaixadas(); desenhar(); } catch (erro) { aviso(String(erro)); }
      }
    };
    document.body.appendChild(p);
  }

  async function baixar(x, arq) {
    const nome = nomeDoArquivo(arq.url);
    progresso[nome] = { feito: 0, total: arq.tamanho || 0 };
    try {
      await chamar('midia_baixar', { url: arq.url, titulo: x.title, imagem: x.images ? imagem(x) : null, tipo: arq.tipo });
      aviso(t('Baixando "{x}"…', { x: x.title }));
      redesenhar();
    } catch (erro) { delete progresso[nome]; aviso(String(erro), 6000); }
  }
  let espera;
  function redesenhar() {
    clearTimeout(espera);
    espera = setTimeout(() => { const p = pilhas[secao]; if (['midia', 'audiobiblia', 'biblioteca'].includes(p[p.length - 1].tela)) desenhar(); }, 400);
  }

  const ouvir = tauri.event?.listen;
  if (ouvir) {
    ouvir('midia-progresso', ({ payload }) => {
      progresso[payload.nome] = { feito: payload.feito, total: payload.total };
      document.querySelectorAll('.cartao-midia .barrinha div').forEach(() => {});
      redesenhar();
    });
    ouvir('midia-fim', async ({ payload }) => {
      delete progresso[payload.nome];
      if (payload.erro) aviso(t('Não deu para baixar: {x}', { x: payload.erro }), 8000);
      else aviso(t('Pronto para ver sem internet.'));
      await carregarBaixadas();
      redesenhar();
    });
  }

  return { montar, categoria, biblia };
})();
