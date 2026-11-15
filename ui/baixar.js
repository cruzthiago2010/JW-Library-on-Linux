// A aba BAIXAR da Biblioteca: escolher publicações no catálogo, baixar do jw.org
// e receber aviso de edição nova ou atualização (o antigo Baixador, embutido).
// Comandos em app/src/baixar.rs; o andamento chega pelos eventos baixar-progresso e baixar-fim.
const Baixar = (() => {
  const estado = {
    lingua: lerLocal('baixar_lingua', 'T'),
    parte: 'catalogo', // catalogo | selecao
    andamento: null, // chave → {feito, total, situacao, erro}
    fim: null,
    novidade: null, // resposta da última procura com algo novo
  };
  function lerLocal(k, padrao) { try { return localStorage.getItem(k) || padrao; } catch (_) { return padrao; } }
  function gravarLocal(k, v) { try { localStorage.setItem(k, v); } catch (_) {} }

  const mb = (b) => (b >= 1073741824 ? `${(b / 1073741824).toFixed(1)} GB` : `${(b / 1048576).toFixed(1)} MB`).replace('.', Idioma.atual === 'en' ? '.' : ',');
  const chaveDe = (p) => (p.edicao ? `${p.sigla}_${p.lingua}_${p.edicao}` : `${p.sigla}_${p.lingua}`);
  const nomeEdicao = (e) => (/^\d{6}$/.test(e) ? new Date(+e.slice(0, 4), +e.slice(4) - 1, 1).toLocaleDateString(Idioma.local, { month: 'short', year: 'numeric' }) : e);
  const capaDe = (par, sigla) => (par ? `<img class="capa" src="${base}/midia/${encodeURIComponent(par[0])}/${encodeURIComponent(par[1])}" alt="">` : `<div class="capa semcapa">${esc(sigla)}</div>`);
  const naTela = () => document.querySelector('.baixar');

  // ---------- Catálogo ----------
  async function catalogo(lugar) {
    const c = await chamar('baixar_catalogo');
    const lingua = estado.lingua;
    const escolha = (sigla) => c.selecao.find((x) => x.pedido.sigla === sigla && x.pedido.lingua === lingua);
    const baixadas = (sigla) => c.baixados.filter((k) => k === `${sigla}_${lingua}` || k.startsWith(`${sigla}_${lingua}_`)).length;
    let html = `<div class="baixar-topo">
      <label>${t('Idioma das publicações')} <select id="lingua">${c.idiomas.map(([k, n]) => `<option value="${k}"${k === lingua ? ' selected' : ''}>${esc(n)}</option>`).join('')}</select></label>
      <span class="dica">${t('Escolha o que quer ter e toque em "Baixar agora" em Minha seleção.')}</span></div>`;
    let categoria = null;
    for (const i of c.itens) {
      if (i.categoria !== categoria) {
        if (categoria) html += '</div>';
        categoria = i.categoria;
        html += `<div class="grupo">${esc(i.nome_da_categoria)}</div><div class="grade">`;
      }
      const e = escolha(i.sigla);
      const n = baixadas(i.sigla);
      html += `<div class="cartao-baixar${e ? ' escolhido' : ''}" data-sigla="${i.sigla}" data-periodico="${i.periodico ? 1 : ''}">
        ${capaDe(c.capas[`${i.sigla}_${lingua}`], i.sigla)}
        <div class="corpo"><div class="t">${esc(i.nome)}</div><div class="m">${i.sigla}${n ? ` · ${n > 1 ? t('{n} edições baixadas', { n }) : t('Baixado')}` : ''}</div>
        ${i.periodico && !e ? `<label class="op"><input type="checkbox" class="novas" checked> ${t('baixar as edições novas')}</label>` : ''}
        ${e ? `<button class="botao leve pequeno" data-o="tirar" data-edicao="${esc(e.pedido.edicao)}">✓ ${t('Na seleção')}</button>` : `<button class="botao pequeno" data-o="escolher">${t('Adicionar')}</button>`}
        </div></div>`;
    }
    html += `</div><div class="grupo">${t('Outra publicação')}</div>
      <div class="outra"><input id="outra-sigla" placeholder="${t('sigla (ex.: g)')}"><input id="outra-edicao" placeholder="${t('edição AAAAMM (se tiver)')}" maxlength="6">
      <button class="botao pequeno" id="outra-ok">${t('Adicionar')}</button><span class="dica">${t('A sigla é a do jw.org.')}</span></div>`;
    lugar.innerHTML = html;
    lugar.querySelector('#lingua').onchange = (x) => { estado.lingua = x.target.value; gravarLocal('baixar_lingua', estado.lingua); desenharAqui(); };
    lugar.querySelectorAll('.cartao-baixar').forEach((cartao) => (cartao.onclick = async (ev) => {
      const b = ev.target.closest('[data-o]');
      if (!b) return;
      const sigla = cartao.dataset.sigla;
      try {
        if (b.dataset.o === 'tirar') await chamar('baixar_tirar', { sigla, lingua, edicao: b.dataset.edicao || '' });
        else await chamar('baixar_escolher', { sigla, lingua, edicao: '', novas: !!cartao.querySelector('.novas')?.checked });
      } catch (erro) { aviso(String(erro)); }
      desenharAqui();
    }));
    lugar.querySelector('#outra-ok').onclick = async () => {
      const sigla = lugar.querySelector('#outra-sigla').value.trim();
      const edicao = lugar.querySelector('#outra-edicao').value.trim();
      if (!sigla) return lugar.querySelector('#outra-sigla').focus();
      if (edicao && !/^\d{6}$/.test(edicao)) return aviso(t('Escreva a edição com 6 números, ex.: 202611.'));
      try { await chamar('baixar_escolher', { sigla, lingua, edicao, novas: false }); aviso(t('{x}: na seleção.', { x: sigla })); } catch (erro) { aviso(String(erro)); }
      desenharAqui();
    };
  }

  // ---------- Minha seleção ----------
  async function selecao(lugar) {
    const [itens, ocupado, op] = await Promise.all([chamar('baixar_selecao'), chamar('baixar_ocupado'), chamar('baixar_opcoes')]);
    if (ocupado && !estado.andamento) estado.andamento = {};
    const rodando = !!estado.andamento && !estado.fim;
    const ultima = op.ultima_verificacao ? new Date(op.ultima_verificacao * 1000).toLocaleString(Idioma.local, { dateStyle: 'short', timeStyle: 'short' }) : t('nunca');
    let html = `<div class="baixar-topo">
      <button class="botao" id="sincronizar"${rodando || !itens.length ? ' disabled' : ''}>${rodando ? t('Baixando…') : t('Baixar agora')}</button>
      <button class="botao leve" id="procurar"${rodando ? ' disabled' : ''}>${t('Procurar atualizações')}</button>
      <label class="op"><input type="checkbox" id="sozinho"${op.atualizar_sozinho ? ' checked' : ''}> ${t('Atualizar sozinho (ao abrir o app, uma vez por dia)')}</label>
      <span class="dica">${t('Última verificação: {x}.', { x: ultima })}</span></div>`;
    if (!itens.length) {
      html += `<div class="vazio"><h2>${t('A seleção está vazia')}</h2><p>${t('Escolha publicações no Catálogo.')}</p></div>`;
    } else {
      html += `<div class="lista">${itens.map((i, k) => {
        const p = i.escolha.pedido;
        const det = [p.sigla, p.lingua];
        if (p.edicao) det.push(t('edição {x}', { x: nomeEdicao(p.edicao) }));
        if (i.escolha.novas_edicoes) det.push(t('edições novas'));
        const situ = i.baixados.length ? (i.escolha.novas_edicoes ? t('{n} baixada(s)', { n: i.baixados.length }) : t('Baixado')) : t('ainda não baixado');
        return `<div class="fila">${capaDe(i.capa, p.sigla)}<div class="corpo"><div class="t">${esc(i.nome)}</div><div class="m">${esc(det.join(' · '))} — ${situ}</div></div>
          <button class="botao leve pequeno" data-tirar="${k}"${rodando ? ' disabled' : ''}>${t('Tirar')}</button></div>`;
      }).join('')}</div>`;
    }
    html += '<div id="andamento"></div>';
    lugar.innerHTML = html;
    desenharAndamento();
    lugar.querySelector('#sincronizar').onclick = sincronizar;
    lugar.querySelector('#procurar').onclick = (x) => procurar(true, x.target);
    lugar.querySelector('#sozinho').onchange = (x) => chamar('baixar_atualizar_sozinho', { valor: x.target.checked }).catch((erro) => aviso(String(erro)));
    lugar.querySelectorAll('[data-tirar]').forEach((b) => (b.onclick = async () => {
      const p = itens[+b.dataset.tirar].escolha.pedido;
      await chamar('baixar_tirar', { sigla: p.sigla, lingua: p.lingua, edicao: p.edicao });
      aviso(t('Saiu da seleção. O que já foi baixado continua na Biblioteca.'));
      desenharAqui();
    }));
  }

  function desenharAndamento() {
    const caixa = document.getElementById('andamento');
    if (!caixa || !estado.andamento) return;
    const linhas = Object.entries(estado.andamento).map(([chave, a]) => {
      const pct = a.total ? Math.round((a.feito * 100) / a.total) : 0;
      const txt = a.erro ? t('erro') : a.situacao === 'JaEstava' ? t('em dia') : a.situacao ? t(a.situacao === 'Atualizado' ? 'atualizado' : 'pronto') : `${pct}% · ${mb(a.feito)} / ${mb(a.total)}`;
      return `<div class="arquivo${a.erro ? ' erro' : ''}" title="${esc(a.erro || '')}"><span class="n">${esc(chave)}</span><div class="barra"><div style="width:${a.situacao || a.erro ? 100 : pct}%"></div></div><span class="q">${txt}</span></div>`;
    }).join('');
    caixa.innerHTML = `<h3>${t(estado.fim ? 'Último download' : 'Baixando')}</h3>${linhas || `<div class="dica">${t('Perguntando ao jw.org…')}</div>`}
      ${estado.fim ? `<div class="fim ${estado.fim.erros ? 'ruim' : 'ok'}">${esc(estado.fim.texto)}</div>${estado.fim.mensagens.map((m) => `<div class="dica">${esc(m)}</div>`).join('')}` : ''}`;
  }

  async function sincronizar() {
    estado.andamento = {};
    estado.fim = null;
    try { await chamar('baixar_sincronizar'); } catch (erro) { estado.andamento = null; aviso(String(erro)); }
    desenharAqui();
  }

  async function atualizarAgora() {
    estado.andamento = {};
    estado.fim = null;
    estado.novidade = null;
    tirarFaixa();
    try { await chamar('baixar_atualizar'); } catch (erro) { estado.andamento = null; aviso(String(erro)); return; }
    estado.parte = 'selecao';
    if (naTela()) desenharAqui();
  }

  function textoDaNovidade(r) {
    const partes = [];
    if (r.versoes) partes.push(r.versoes === 1 ? t('1 publicação tem versão nova') : t('{n} publicações têm versão nova', { n: r.versoes }));
    if (r.novas) partes.push(r.novas === 1 ? t('1 publicação nova para baixar') : t('{n} publicações novas para baixar', { n: r.novas }));
    return partes.join(t(' e '));
  }

  async function procurar(forcar, botao) {
    if (botao) { botao.disabled = true; botao.textContent = t('Procurando…'); }
    try {
      const r = await chamar('baixar_procurar', { forcar });
      if (r && (r.versoes || r.novas)) {
        if (r.sozinho && !forcar) { aviso(`${textoDaNovidade(r)}. ${t('Atualizando sozinho…')}`, 6000); await atualizarAgora(); }
        else { estado.novidade = r; mostrarFaixa(); }
      } else if (r && forcar) {
        aviso(r.erros.length ? t('Não consegui conferir: {x}', { x: r.erros[0] }) : t('Tudo em dia.'));
      }
    } catch (erro) { if (forcar) aviso(String(erro)); }
    if (botao && naTela()) desenharAqui();
  }

  // ---------- Aviso de novidade (no Início e na Biblioteca) ----------
  function tirarFaixa() { document.querySelectorAll('.faixa-novidade').forEach((f) => f.remove()); }
  function mostrarFaixa() {
    tirarFaixa();
    const r = estado.novidade;
    if (!r) return;
    const tela = document.querySelector('#conteudo .tela');
    if (!tela) return;
    const nomes = [...new Set(r.novidades.map((n) => n.novo.publicacao + (n.pedido.edicao ? ` (${nomeEdicao(n.pedido.edicao)})` : '')))];
    const f = document.createElement('div');
    f.className = 'faixa-novidade';
    f.innerHTML = `${icone('nuvem')}<div class="texto"><b>${esc(textoDaNovidade(r))}</b><span>${esc(nomes.join(' · '))}</span></div>
      <button class="botao">${t('Atualizar')}</button><button class="fechar-faixa" title="${t('Fechar')}">×</button>`;
    f.querySelector('.botao').onclick = atualizarAgora;
    f.querySelector('.fechar-faixa').onclick = () => { estado.novidade = null; f.remove(); };
    tela.prepend(f);
  }

  // ---------- Eventos ----------
  const ouvir = tauri.event?.listen;
  if (ouvir) {
    ouvir('baixar-progresso', ({ payload }) => {
      if (!estado.andamento) estado.andamento = {};
      estado.andamento[payload.chave] = { ...(estado.andamento[payload.chave] || {}), feito: payload.feito, total: payload.total };
      desenharAndamento();
    });
    ouvir('baixar-fim', async ({ payload }) => {
      if (!estado.andamento) estado.andamento = {};
      let feitos = 0, erros = 0;
      const mensagens = [];
      if (payload.erro) { erros++; mensagens.push(payload.erro); }
      for (const d of payload.desfechos || []) {
        if (d.Feito) {
          const r = d.Feito;
          const chave = Object.keys(estado.andamento).find((k) => r.pasta.startsWith(k) || k === r.pasta) || r.pasta;
          estado.andamento[chave] = { ...(estado.andamento[chave] || { feito: r.arquivo.tamanho, total: r.arquivo.tamanho }), situacao: r.situacao };
          if (r.situacao !== 'JaEstava') feitos++;
        } else if (d.Erro) {
          const [pedido, erro] = d.Erro;
          const chave = chaveDe(pedido);
          estado.andamento[chave] = { ...(estado.andamento[chave] || { feito: 0, total: 0 }), erro };
          mensagens.push(`${chave}: ${erro}`);
          erros++;
        }
      }
      const texto = erros ? t('{n} baixada(s), {e} com erro.', { n: feitos, e: erros })
        : feitos ? t('Pronto: {n} publicação(ões) na Biblioteca.', { n: feitos }) : t('Tudo em dia. Nada novo para baixar.');
      estado.fim = { texto, erros, mensagens };
      aviso(texto, 5000);
      try { await carregarBiblioteca(); } catch (_) {}
      if (naTela()) desenharAqui();
    });
  }

  // ---------- Montar ----------
  let lugarAtual = null;
  async function desenharAqui() {
    if (!lugarAtual || !document.body.contains(lugarAtual)) return;
    lugarAtual.querySelectorAll('[data-parte]').forEach((b) => b.classList.toggle('ativa', b.dataset.parte === estado.parte));
    const corpo = lugarAtual.querySelector('.baixar-corpo');
    try { await (estado.parte === 'selecao' ? selecao : catalogo)(corpo); }
    catch (erro) { corpo.innerHTML = `<div class="vazio"><h2>${t('Não deu para abrir')}</h2><p>${esc(erro)}</p></div>`; }
  }
  function montar(lugar) {
    lugarAtual = lugar;
    lugar.innerHTML = `<div class="baixar"><div class="sub-abas">${[['catalogo', 'Catálogo'], ['selecao', 'Minha seleção']].map(([k, n]) =>
      `<button data-parte="${k}">${t(n)}</button>`).join('')}</div><div class="baixar-corpo"></div>
      <p class="dica aviso-termos">${t('Baixa direto do jw.org, uma pergunta por segundo. Para uso pessoal.')}</p></div>`;
    lugar.querySelectorAll('[data-parte]').forEach((b) => (b.onclick = () => { estado.parte = b.dataset.parte; desenharAqui(); }));
    desenharAqui();
  }

  return { montar, procurarAoAbrir: () => procurar(false), mostrarFaixa };
})();
