// Grifos, notas, campos de resposta e marcadores no leitor (Etapa 4).
//
// Os tokens seguem a regra do JW Library (a mesma de nucleo/src/tokens.rs):
// base 0, fim inclusivo; o número do parágrafo/versículo e as letras de
// referência não contam; pontuação no começo/fim da palavra é token próprio;
// dois pedaços de texto colados (sem espaço) são tokens separados.
// Os grifos são pintados com a API de destaques do CSS (::highlight), sem mexer
// no HTML da publicação — assim os tokens nunca mudam de lugar.
const Marcas = (() => {
  const PONTUACAO = '.,;:!?()[]“”‘’"\'—–…/';
  const PULAR = '.parNum, .vl, .cl, .m, .tt, .gen-field, .nota-marca, textarea, label, .groupFootnote';
  // Ordem das cores na barra, como no JW Library: amarelo, verde, azul, roxo, rosa, laranja.
  const CORES = [1, 2, 3, 6, 4, 5];
  const NOMES = { 1: 'Amarelo', 2: 'Verde', 3: 'Azul', 4: 'Rosa', 5: 'Laranja', 6: 'Roxo' };

  let ctx = null; // { c, alvo, biblia, livro, capitulo, dados, blocos: Map }

  // ---------------------------------------------------------------- tokens
  function elementosDoBloco(id) {
    const doc = ctx.c.querySelector('.documento');
    if (ctx.biblia) return [...doc.querySelectorAll(`[id^="v${ctx.livro}-${ctx.capitulo}-${id}-"]`)];
    const p = doc.querySelector(`[data-pid="${id}"]`);
    return p ? [p] : [];
  }
  function blocoDoNo(no) {
    const el = no.nodeType === 1 ? no : no.parentElement;
    if (!el) return null;
    if (ctx.biblia) {
      const v = el.closest('.v[id]');
      return v ? +v.id.split('-')[2] : null;
    }
    const p = el.closest('[data-pid]');
    if (!p || p.classList.contains('gen-field') || p.querySelector('[data-pid]')) return null;
    return +p.dataset.pid;
  }
  function tokens(id) {
    if (ctx.blocos.has(id)) return ctx.blocos.get(id);
    const lista = [];
    let anterior = ''; // último caractere juntado
    for (const raiz of elementosDoBloco(id)) {
      const andar = document.createTreeWalker(raiz, NodeFilter.SHOW_TEXT);
      for (let no = andar.nextNode(); no; no = andar.nextNode()) {
        if (no.parentElement.closest(PULAR)) continue;
        const t = no.data;
        if (!t) continue;
        const limite = anterior && !/\s/.test(anterior) && !/\s/.test(t[0]);
        // Palavras deste nó; o limite só importa porque separa (já separa por nó).
        void limite;
        let i = 0;
        while (i < t.length) {
          if (/\s/.test(t[i])) { i++; continue; }
          const base = i;
          while (i < t.length && !/\s/.test(t[i])) i++;
          let a = base, b = i;
          while (a < b && PONTUACAO.includes(t[a])) { lista.push({ no, a, b: a + 1 }); a++; }
          const cauda = [];
          while (b > a && PONTUACAO.includes(t[b - 1])) { b--; cauda.push({ no, a: b, b: b + 1 }); }
          if (a < b) lista.push({ no, a, b });
          lista.push(...cauda.reverse());
        }
        anterior = t[t.length - 1];
      }
    }
    ctx.blocos.set(id, lista);
    return lista;
  }
  function faixa(id, de, ate) {
    const t = tokens(id);
    if (!t.length) return null;
    const a = t[Math.max(0, de)], b = t[Math.min(ate, t.length - 1)];
    if (!a || !b) return null;
    const r = document.createRange();
    r.setStart(a.no, a.a);
    r.setEnd(b.no, b.b);
    return r;
  }
  // O token num ponto (nó, posição): o que contém o ponto ou, entre palavras,
  // o seguinte (para o começo) ou o anterior (para o fim).
  function tokenNoPonto(id, no, pos, comeco) {
    const t = tokens(id);
    const r = document.createRange();
    let ultimoAntes = -1;
    for (let i = 0; i < t.length; i++) {
      r.setStart(t[i].no, t[i].a);
      r.setEnd(t[i].no, t[i].b);
      const cmp = r.comparePoint(no, pos);
      if (cmp === 0) {
        if (!comeco && pos === t[i].a && no === t[i].no) return i - 1 >= 0 ? i - 1 : 0;
        return i;
      }
      if (cmp < 0) return comeco ? i : ultimoAntes;
      ultimoAntes = i;
    }
    return comeco ? -1 : ultimoAntes;
  }
  function textoDosTokens(id, de, ate) {
    const r = faixa(id, de, ate);
    return r ? r.toString().replace(/\s+/g, ' ').trim() : '';
  }
  function ordemDosBlocos() {
    const doc = ctx.c.querySelector('.documento');
    if (ctx.biblia) return [...new Set([...doc.querySelectorAll('.v[id]')].map((v) => +v.id.split('-')[2]))];
    return [...doc.querySelectorAll('[data-pid]')].filter((p) => !p.classList.contains('gen-field') && !p.querySelector('[data-pid]')).map((p) => +p.dataset.pid);
  }

  // ---------------------------------------------------------------- pintar
  function pintar() {
    if (!window.Highlight || !CSS.highlights) return;
    for (const cor of [1, 2, 3, 4, 5, 6]) CSS.highlights.delete('grifo' + cor);
    CSS.highlights.delete('selecao-grifo');
    const porCor = {};
    for (const g of ctx.dados.grifos) {
      for (const f of g.faixas) {
        const r = faixa(f.bloco, f.inicio, f.fim);
        if (r) (porCor[g.cor] = porCor[g.cor] || []).push(r);
      }
    }
    for (const [cor, ranges] of Object.entries(porCor)) CSS.highlights.set('grifo' + cor, new Highlight(...ranges));
    // Marca de nota na margem do bloco.
    ctx.c.querySelectorAll('.nota-marca').forEach((m) => m.remove());
    const comNota = new Map();
    for (const n of ctx.dados.notas) if (n.bloco != null) comNota.set(n.bloco, n.cor);
    for (const [bloco, cor] of comNota) {
      const el = elementosDoBloco(bloco)[0];
      if (!el) continue;
      const m = document.createElement('span');
      m.className = 'nota-marca cor-' + (cor || 0);
      m.title = 'Tem nota';
      m.dataset.bloco = bloco;
      el.prepend(m);
    }
  }

  // ---------------------------------------------------------------- barra flutuante
  function fecharBarra() {
    document.querySelectorAll('.barra-grifo').forEach((b) => b.remove());
    if (CSS.highlights) CSS.highlights.delete('selecao-grifo');
  }
  function abrirBarra(retangulo, sel, grifo) {
    fecharBarra();
    const b = document.createElement('div');
    b.className = 'barra-grifo';
    b.innerHTML = `${CORES.map((k) => `<button class="cor cor-${k}${grifo && grifo.cor === k ? ' atual' : ''}" data-cor="${k}" title="${NOMES[k]}">${grifo && grifo.cor === k ? '✓' : ''}</button>`).join('')}
      <span class="sep"></span>
      <button data-o="nota" title="Adicionar uma nota">${icone('nota')}</button>
      ${grifo ? `<button data-o="apagar" title="Tirar o grifo">${icone('lixeira')}</button>` : ''}
      <button data-o="copiar" title="Copiar">${icone('copiar')}</button>`;
    document.body.appendChild(b);
    const topo = retangulo.top - b.offsetHeight - 10;
    b.style.top = (topo < 90 ? retangulo.bottom + 10 : topo) + 'px';
    b.style.left = Math.min(window.innerWidth - b.offsetWidth - 10, Math.max(56, retangulo.left + retangulo.width / 2 - b.offsetWidth / 2)) + 'px';
    b.addEventListener('mousedown', (e) => e.preventDefault()); // não perde a seleção
    b.onclick = async (e) => {
      const alvoBotao = e.target.closest('button');
      if (!alvoBotao) return;
      try {
        if (alvoBotao.dataset.cor) {
          await chamar('grifar', { alvo: ctx.alvo, faixas: grifo ? grifo.faixas : sel.faixas, cor: +alvoBotao.dataset.cor, guid: grifo ? grifo.guid : null });
        } else if (alvoBotao.dataset.o === 'apagar') {
          await chamar('remover_grifo', { guid: grifo.guid });
        } else if (alvoBotao.dataset.o === 'copiar') {
          const texto = grifo ? grifo.faixas.map((f) => textoDosTokens(f.bloco, f.inicio, f.fim)).join(' ') : sel.texto;
          try { await navigator.clipboard.writeText(texto); aviso('Copiado.'); } catch (_) { aviso('Não deu para copiar.'); }
        } else if (alvoBotao.dataset.o === 'nota') {
          const faixas = grifo ? grifo.faixas : sel.faixas;
          await novaNota(faixas[0].bloco, grifo ? grifo.guid : null);
        }
      } catch (erro) { aviso(String(erro)); }
      window.getSelection()?.removeAllRanges();
      fecharBarra();
      await recarregar();
    };
  }
  function lerSelecao() {
    const s = window.getSelection();
    if (!s || s.isCollapsed || !s.rangeCount) return null;
    const r = s.getRangeAt(0);
    const doc = ctx.c.querySelector('.documento');
    if (!doc.contains(r.commonAncestorContainer)) return null;
    let ini = blocoDoNo(r.startContainer), fim = blocoDoNo(r.endContainer);
    const ordem = ordemDosBlocos();
    if (ini == null) ini = ordem.find((b) => elementosDoBloco(b).some((el) => r.intersectsNode(el)));
    if (fim == null) fim = [...ordem].reverse().find((b) => elementosDoBloco(b).some((el) => r.intersectsNode(el)));
    if (ini == null || fim == null) return null;
    const i0 = ordem.indexOf(ini), i1 = ordem.indexOf(fim);
    const faixas = [];
    for (let i = i0; i <= i1; i++) {
      const b = ordem[i];
      const total = tokens(b).length;
      if (!total) continue;
      const de = b === ini ? tokenNoPonto(b, r.startContainer, r.startOffset, true) : 0;
      const ate = b === fim ? tokenNoPonto(b, r.endContainer, r.endOffset, false) : total - 1;
      if (de < 0 || ate < de) continue;
      faixas.push({ bloco: b, inicio: de, fim: ate });
    }
    if (!faixas.length) return null;
    return { faixas, texto: s.toString().trim(), retangulo: r.getBoundingClientRect() };
  }
  function grifoNoPonto(x, y) {
    const pos = document.caretRangeFromPoint ? document.caretRangeFromPoint(x, y) : null;
    if (!pos) return null;
    const bloco = blocoDoNo(pos.startContainer);
    if (bloco == null) return null;
    const t = tokenNoPonto(bloco, pos.startContainer, pos.startOffset, true);
    if (t < 0) return null;
    return ctx.dados.grifos.find((g) => g.faixas.some((f) => f.bloco === bloco && f.inicio <= t && t <= f.fim)) || null;
  }

  // ---------------------------------------------------------------- notas no painel
  function cartaoDeNota(n) {
    const rotulo = n.bloco == null ? 'Nota' : ctx.biblia ? `${ctx.capitulo}:${n.bloco}` : `Parágrafo ${numeroDoParagrafo(n.bloco)}`;
    return `<div class="nota-cartao cor-${n.cor || 0}" data-guid="${n.guid}">
      <div class="nota-cab"><span>${esc(rotulo)}</span><button data-o="apagar" title="Excluir a nota">${icone('lixeira')}</button></div>
      <input class="nota-titulo" placeholder="Título" value="${esc(n.titulo)}">
      <textarea class="nota-texto" placeholder="Nota" rows="3">${esc(n.texto)}</textarea>
      <input class="nota-etiquetas" placeholder="+ Adicionar uma etiqueta (separe por vírgula)" value="${esc(n.etiquetas.join(', '))}">
    </div>`;
  }
  function numeroDoParagrafo(pid) {
    const n = elementosDoBloco(pid)[0]?.querySelector('.parNum')?.textContent?.replace(/\D/g, '');
    return n || pid;
  }
  function desenharNotas() {
    const lugar = ctx.c.querySelector('.minhas-notas');
    if (!lugar) return;
    const notas = ctx.dados.notas;
    lugar.innerHTML = notas.length ? `<div class="paragrafo">Minhas notas</div>${notas.map(cartaoDeNota).join('')}` : '';
    lugar.querySelectorAll('.nota-cartao').forEach((cartao) => {
      const guid = cartao.dataset.guid;
      const nota = notas.find((n) => n.guid === guid);
      let espera;
      const salvar = () => {
        clearTimeout(espera);
        espera = setTimeout(async () => {
          const etiquetas = cartao.querySelector('.nota-etiquetas').value.split(',').map((x) => x.trim()).filter(Boolean);
          await chamar('anotar', { alvo: ctx.alvo, guid, bloco: nota.bloco, grifo: nota.grifo, titulo: cartao.querySelector('.nota-titulo').value, texto: cartao.querySelector('.nota-texto').value, etiquetas });
        }, 500);
      };
      cartao.querySelectorAll('input, textarea').forEach((x) => (x.oninput = salvar));
      cartao.querySelector('[data-o="apagar"]').onclick = () => {
        dialogo('Excluir nota', '<p>Isso não poderá ser desfeito.</p>', [['Cancelar', null], ['Excluir', async () => { await chamar('apagar_nota', { guid }); await recarregar(); }]]);
      };
    });
  }
  async function novaNota(bloco, grifo) {
    const guid = await chamar('anotar', { alvo: ctx.alvo, guid: null, bloco, grifo, titulo: '', texto: '', etiquetas: [] });
    await recarregar();
    abrirPainel(ctx.c);
    const cartao = ctx.c.querySelector(`.nota-cartao[data-guid="${guid}"]`);
    cartao?.scrollIntoView({ block: 'center' });
    cartao?.querySelector('.nota-titulo').focus();
  }

  // ---------------------------------------------------------------- campos de resposta
  function ligarCampos() {
    const valores = Object.fromEntries(ctx.dados.campos);
    ctx.c.querySelectorAll('.documento textarea[id]').forEach((t) => {
      if (valores[t.id] != null) t.value = valores[t.id];
      let espera;
      t.oninput = () => {
        clearTimeout(espera);
        espera = setTimeout(() => chamar('campo', { alvo: ctx.alvo, tag: t.id, valor: t.value }).catch((e) => aviso(String(e))), 500);
      };
    });
  }

  // ---------------------------------------------------------------- marcadores
  async function marcadores() {
    const lista = await chamar('marcadores', { alvo: ctx.alvo });
    const porVaga = Object.fromEntries(lista.map((m) => [m.slot, m]));
    const corpo = `<div class="fitas">${Array.from({ length: 10 }, (_, i) => {
      const m = porVaga[i];
      return `<div class="fita fita-${i}"><button class="fita-ir" data-vaga="${i}">${m ? `<b>${esc(m.titulo)}</b><small>${esc(m.trecho)}</small>` : '<span class="nao-usado">Não usado — clique para marcar aqui</span>'}</button>
        ${m ? `<button class="fita-x" data-tirar="${i}" title="Tirar o marcador">×</button>` : ''}</div>`;
    }).join('')}</div>`;
    const f = dialogo('Marcadores', corpo, [['Concluído', null]]);
    f.querySelectorAll('[data-vaga]').forEach((b) => (b.onclick = async () => {
      const m = porVaga[+b.dataset.vaga];
      f.remove();
      if (m) return irPara(m);
      const bloco = primeiroBlocoVisivel();
      const trecho = bloco != null ? textoDosTokens(bloco, 0, 14) : '';
      await chamar('marcar', { alvo: ctx.alvo, slot: +b.dataset.vaga, bloco, trecho });
      aviso('Marcador colocado.');
    }));
    f.querySelectorAll('[data-tirar]').forEach((b) => (b.onclick = async (e) => {
      e.stopPropagation();
      await chamar('desmarcar', { alvo: ctx.alvo, slot: +b.dataset.tirar });
      f.remove();
      marcadores();
    }));
  }
  function primeiroBlocoVisivel() {
    const topo = ctx.c.getBoundingClientRect().top + 80;
    for (const b of ordemDosBlocos()) {
      const el = elementosDoBloco(b)[0];
      if (el && el.getBoundingClientRect().bottom > topo) return b;
    }
    return null;
  }
  async function irPara(m) {
    if (m.livro) return abrir({ tela: 'capitulo', livro: m.livro, capitulo: m.capitulo, verso: m.bloco });
    const achado = m.meps ? await chamar('achar_meps', { meps: m.meps }) : null;
    if (achado) abrir({ tela: 'documento', pasta: achado[0], id: achado[1], bloco: m.bloco });
    else aviso('Essa publicação não está na sua biblioteca.');
  }

  // ---------------------------------------------------------------- montar
  async function recarregar() {
    if (!ctx) return;
    ctx.dados = await chamar('marcas_do_lugar', { alvo: ctx.alvo });
    pintar();
    desenharNotas();
  }
  async function montar(c, alvo) {
    fecharBarra();
    ctx = { c, alvo, biblia: alvo.tipo === 'biblia', livro: alvo.livro, capitulo: alvo.capitulo, blocos: new Map(), dados: { grifos: [], notas: [], campos: [] } };
    const corpo = c.querySelector('.painel-corpo');
    if (corpo && !c.querySelector('.minhas-notas')) corpo.insertAdjacentHTML('beforebegin', '<div class="minhas-notas"></div>');
    await recarregar();
    ligarCampos();
    const doc = c.querySelector('.documento');
    doc.addEventListener('mouseup', (e) => {
      if (e.button !== 0 || e.target.closest('textarea, a, .vl, .cl, .m')) return;
      setTimeout(() => {
        const sel = lerSelecao();
        if (sel) return abrirBarra(sel.retangulo, sel, null);
        const g = grifoNoPonto(e.clientX, e.clientY);
        if (g) {
          const r = faixa(g.faixas[0].bloco, g.faixas[0].inicio, g.faixas[0].fim);
          if (r && CSS.highlights) CSS.highlights.set('selecao-grifo', new Highlight(r));
          abrirBarra(r ? r.getBoundingClientRect() : { top: e.clientY, bottom: e.clientY, left: e.clientX, width: 0 }, null, g);
        } else fecharBarra();
      }, 10);
    });
    doc.addEventListener('click', (e) => {
      const m = e.target.closest('.nota-marca');
      if (!m) return;
      abrirPainel(c);
      const nota = ctx.dados.notas.find((n) => String(n.bloco) === m.dataset.bloco);
      c.querySelector(`.nota-cartao[data-guid="${nota?.guid}"]`)?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    });
    c.addEventListener('scroll', fecharBarra, { passive: true });
  }
  document.addEventListener('mousedown', (e) => { if (!e.target.closest('.barra-grifo')) fecharBarra(); });

  return { montar, marcadores, novaNota, recarregar, textoDosTokens, get ativo() { return ctx; } };
})();
