// A janela do baixador: três abas (Catálogo, Minha seleção, Baixados).
// Fala com o Rust por window.__TAURI__ (comandos em src/janela.rs).
"use strict";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const conteudo = document.getElementById("conteudo");
const seletorIdioma = document.getElementById("idioma");
const caixaIdioma = document.getElementById("idioma-caixa");

const estado = {
  aba: "catalogo",
  lingua: lerPreferencia("lingua", "T"),
  idiomas: [],
  // Arquivos da sincronização atual: chave → {feito, total, situacao, erro}
  andamento: null,
  fim: null,
  confirmando: null,
};

function lerPreferencia(nome, padrao) {
  try { return localStorage.getItem(nome) || padrao; } catch { return padrao; }
}
function gravarPreferencia(nome, valor) {
  try { localStorage.setItem(nome, valor); } catch { /* sem armazenamento: tudo bem */ }
}

function el(tag, atributos = {}, ...filhos) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(atributos)) {
    if (v === undefined || v === null || v === false) continue;
    if (k.startsWith("on")) e.addEventListener(k.slice(2), v);
    else if (k === "class") e.className = v;
    else e.setAttribute(k, v === true ? "" : v);
  }
  for (const f of filhos.flat()) {
    if (f === null || f === undefined || f === false) continue;
    e.append(f instanceof Node ? f : document.createTextNode(String(f)));
  }
  return e;
}

let avisoTempo;
function avisar(texto) {
  const a = document.getElementById("aviso");
  a.textContent = texto;
  a.classList.add("visivel");
  clearTimeout(avisoTempo);
  avisoTempo = setTimeout(() => a.classList.remove("visivel"), 3500);
}

const mb = (b) => (b >= 1073741824 ? `${(b / 1073741824).toFixed(1)} GB` : `${(b / 1048576).toFixed(1)} MB`).replace(".", ",");
const data = (s) => new Date(s * 1000).toLocaleDateString("pt-BR", { day: "2-digit", month: "short", year: "numeric" });
const nomeIdioma = (c) => (estado.idiomas.find((i) => i.codigo === c) || { nome: c }).nome;

function capa(url, sigla) {
  if (url) {
    const img = el("img", { class: "capa", src: url, alt: "", loading: "lazy" });
    img.addEventListener("error", () => img.replaceWith(el("div", { class: "capa semcapa" }, sigla)));
    return img;
  }
  return el("div", { class: "capa semcapa" }, sigla);
}

// Edição "202611" → "nov. 2026".
function nomeEdicao(e) {
  if (!/^\d{6}$/.test(e)) return e;
  const d = new Date(Number(e.slice(0, 4)), Number(e.slice(4)) - 1, 1);
  return d.toLocaleDateString("pt-BR", { month: "short", year: "numeric" });
}

// ---------- Catálogo ----------

async function telaCatalogo() {
  const c = await invoke("catalogo");
  estado.idiomas = c.idiomas;
  preencherIdiomas();
  const lingua = estado.lingua;
  const naSelecao = (sigla) => c.selecao.find((e) => e.pedido.sigla === sigla && e.pedido.lingua === lingua);
  const baixou = (sigla) => c.baixados.filter((k) => k === `${sigla}_${lingua}` || k.startsWith(`${sigla}_${lingua}_`));

  const tela = el("div", { class: "tela" });
  tela.append(outraSigla());

  let categoria = null;
  let grade = null;
  for (const item of c.itens) {
    if (item.categoria !== categoria) {
      categoria = item.categoria;
      tela.append(el("div", { class: "grupo" }, item.nome_da_categoria));
      grade = el("div", { class: "grade" });
      tela.append(grade);
    }
    grade.append(cartaoDoCatalogo(item, naSelecao(item.sigla), baixou(item.sigla), c.capas[`${item.sigla}_${lingua}`]));
  }
  return tela;
}

function cartaoDoCatalogo(item, escolha, baixados, urlCapa) {
  const lingua = estado.lingua;
  const corpo = el("div", { class: "corpo" },
    el("div", { class: "t" }, item.nome),
    el("div", { class: "m" }, `${item.sigla} · ${nomeIdioma(lingua)}`));

  let novas = escolha ? escolha.novas_edicoes : true;
  let campoEdicao = null;
  if (item.periodico) {
    const marca = el("input", { type: "checkbox", checked: novas });
    campoEdicao = el("input", { class: "edicao", placeholder: "edição AAAAMM", inputmode: "numeric", maxlength: "6" });
    campoEdicao.hidden = novas;
    marca.addEventListener("change", async () => {
      novas = marca.checked;
      campoEdicao.hidden = novas;
      // Já escolhido sem edição fixa: troca a opção na hora.
      if (escolha && !escolha.pedido.edicao && novas) {
        await invoke("escolher", { sigla: item.sigla, lingua, edicao: "", novas: true });
        mostrar();
      }
    });
    corpo.append(el("label", { class: "opcao" }, marca, "baixar edições novas"), campoEdicao);
  }

  const linha = el("div", { class: "linha" });
  if (escolha) {
    linha.append(el("button", {
      class: "botao leve pequeno", title: "Clique para tirar da seleção",
      onclick: async () => {
        await invoke("tirar", { sigla: item.sigla, lingua, edicao: escolha.pedido.edicao });
        avisar(`${item.nome} saiu da seleção.`);
        mostrar();
      },
    }, "✓ Na seleção"));
  } else {
    linha.append(el("button", {
      class: "botao pequeno",
      onclick: async () => {
        let edicao = "";
        if (item.periodico && !novas) {
          edicao = campoEdicao.value.trim();
          if (!/^\d{6}$/.test(edicao)) { avisar("Escreva a edição com 6 números, ex.: 202611."); campoEdicao.focus(); return; }
        }
        await invoke("escolher", { sigla: item.sigla, lingua, edicao, novas: item.periodico && novas });
        avisar(`${item.nome}: na seleção.`);
        mostrar();
      },
    }, "Adicionar"));
  }
  if (baixados.length) {
    linha.append(el("span", { class: "marca" }, baixados.length > 1 ? `${baixados.length} edições baixadas` : "Baixado"));
  }
  corpo.append(linha);
  return el("div", { class: `cartao${escolha ? " escolhido" : ""}` }, capa(urlCapa, item.sigla), corpo);
}

function outraSigla() {
  const sigla = el("input", { placeholder: "sigla (ex.: g)", "aria-label": "Sigla" });
  const edicao = el("input", { placeholder: "edição (se tiver)", "aria-label": "Edição", inputmode: "numeric", maxlength: "6" });
  const adicionar = async () => {
    const s = sigla.value.trim().toLowerCase();
    if (!s) { sigla.focus(); return; }
    try {
      await invoke("escolher", { sigla: s, lingua: estado.lingua, edicao: edicao.value.trim(), novas: false });
      avisar(`${s}: na seleção. Se a sigla não existir, a sincronização avisa.`);
      mostrar();
    } catch (e) { avisar(String(e)); }
  };
  sigla.addEventListener("keydown", (e) => { if (e.key === "Enter") adicionar(); });
  edicao.addEventListener("keydown", (e) => { if (e.key === "Enter") adicionar(); });
  return el("div", { class: "outra" }, el("strong", {}, "Outra publicação"), sigla, edicao,
    el("button", { class: "botao pequeno", onclick: adicionar }, "Adicionar"),
    el("span", { class: "suave" }, "A sigla é a do jw.org, no idioma escolhido acima."));
}

function preencherIdiomas() {
  if (seletorIdioma.options.length) return;
  for (const i of estado.idiomas) seletorIdioma.append(el("option", { value: i.codigo }, i.nome));
  if (!estado.idiomas.some((i) => i.codigo === estado.lingua)) seletorIdioma.append(el("option", { value: estado.lingua }, estado.lingua));
  seletorIdioma.value = estado.lingua;
}

seletorIdioma.addEventListener("change", () => {
  estado.lingua = seletorIdioma.value;
  gravarPreferencia("lingua", estado.lingua);
  mostrar();
});

// ---------- Minha seleção ----------

async function telaSelecao() {
  const [itens, ocupado] = await Promise.all([invoke("selecao"), invoke("sincronizando")]);
  if (ocupado && !estado.andamento) estado.andamento = {};
  const tela = el("div", { class: "tela" });
  const rodando = !!estado.andamento && !estado.fim;
  const botao = el("button", { class: "botao grande", disabled: rodando || !itens.length, onclick: sincronizar },
    rodando ? "Sincronizando…" : "Sincronizar agora");
  const verificar = el("button", { class: "botao leve", disabled: rodando, onclick: () => procurarAtualizacoes(verificar) }, "Procurar atualizações");
  tela.append(el("div", { class: "topo" }, botao, verificar,
    el("div", { class: "resumo" }, itens.length
      ? "Baixa o que falta e atualiza o que mudou. As publicações vão direto para a Biblioteca do leitor."
      : "")));

  if (!itens.length) {
    tela.append(el("div", { class: "vazio" }, el("h2", {}, "A seleção está vazia"),
      el("p", {}, "Escolha publicações no Catálogo."),
      el("button", { class: "botao", onclick: () => trocarAba("catalogo") }, "Abrir o Catálogo")));
  } else {
    const lista = el("div", { class: "lista" });
    for (const i of itens) {
      const p = i.escolha.pedido;
      const detalhes = [p.sigla, nomeIdioma(p.lingua)];
      if (p.edicao) detalhes.push(`edição ${nomeEdicao(p.edicao)}`);
      if (i.escolha.novas_edicoes) detalhes.push("edições novas");
      const baixados = i.baixados.length
        ? (i.escolha.novas_edicoes ? `${i.baixados.length} ${i.baixados.length === 1 ? "edição baixada" : "edições baixadas"}` : "baixado")
        : "ainda não baixado";
      lista.append(el("div", { class: "fila" }, capa(i.capa, p.sigla),
        el("div", { class: "corpo" }, el("div", { class: "t" }, i.nome), el("div", { class: "m" }, `${detalhes.join(" · ")} — ${baixados}`)),
        el("div", { class: "acoes" }, el("button", {
          class: "botao leve pequeno", disabled: rodando,
          onclick: async () => {
            await invoke("tirar", { sigla: p.sigla, lingua: p.lingua, edicao: p.edicao });
            avisar(`${i.nome} saiu da seleção. O que já foi baixado continua em Baixados.`);
            mostrar();
          },
        }, "Remover"))));
    }
    tela.append(lista);
  }
  if (estado.andamento) tela.append(desenharAndamento());
  return tela;
}

function desenharAndamento() {
  const caixa = el("div", { class: "andamento", id: "andamento" });
  caixa.append(el("h3", {}, estado.fim ? "Última sincronização" : "Sincronizando"));
  const chaves = Object.keys(estado.andamento);
  if (!chaves.length && !estado.fim) caixa.append(el("div", { class: "suave" }, "Perguntando ao jw.org…"));
  for (const chave of chaves) {
    const a = estado.andamento[chave];
    const pct = a.total ? Math.round((a.feito * 100) / a.total) : 0;
    const texto = a.erro ? "erro" : a.situacao === "JaEstava" ? "em dia" : a.situacao ? (a.situacao === "Atualizado" ? "atualizado" : "pronto")
      : `${pct}% · ${mb(a.feito)} de ${mb(a.total)}`;
    const barra = el("div", { class: "barra" }, el("div", { style: `width:${a.situacao || a.erro ? 100 : pct}%` }));
    caixa.append(el("div", { class: `arquivo${a.situacao ? " pronto" : ""}${a.erro ? " erro" : ""}`, title: a.erro || "" },
      el("span", { class: "n" }, chave), barra, el("span", { class: "q" }, texto)));
  }
  if (estado.fim) {
    const f = estado.fim;
    caixa.append(el("div", { class: `fim ${f.erros ? "ruim" : "ok"}` }, f.texto));
    for (const e of f.mensagens) caixa.append(el("div", { class: "suave" }, e));
  }
  return caixa;
}

function atualizarAndamento() {
  const velho = document.getElementById("andamento");
  if (velho && estado.aba === "selecao") velho.replaceWith(desenharAndamento());
}

async function sincronizar() {
  estado.andamento = {};
  estado.fim = null;
  try {
    await invoke("sincronizar");
  } catch (e) {
    estado.andamento = null;
    avisar(String(e));
  }
  mostrar();
}

listen("progresso", ({ payload }) => {
  if (!estado.andamento) estado.andamento = {};
  const atual = estado.andamento[payload.chave] || {};
  estado.andamento[payload.chave] = { ...atual, feito: payload.feito, total: payload.total };
  if (!document.getElementById("andamento") && estado.aba === "selecao") mostrar();
  else atualizarAndamento();
});

listen("fim", ({ payload }) => {
  if (!estado.andamento) estado.andamento = {};
  let feitos = 0, erros = 0;
  const mensagens = [];
  if (payload.erro) { erros = 1; mensagens.push(payload.erro); }
  for (const d of payload.desfechos || []) {
    if (d.Feito) {
      const r = d.Feito;
      // A chave do registro é a do pedido; a pasta pode ter outro nome.
      const chave = Object.keys(estado.andamento).find((k) => k === r.pasta) || r.pasta;
      const atual = estado.andamento[chave] || { feito: r.arquivo.tamanho, total: r.arquivo.tamanho };
      estado.andamento[chave] = { ...atual, situacao: r.situacao };
      if (r.situacao !== "JaEstava") feitos++;
    } else if (d.Erro) {
      const [pedido, erro] = d.Erro;
      const chave = chaveDe(pedido);
      estado.andamento[chave] = { ...(estado.andamento[chave] || { feito: 0, total: 0 }), erro };
      mensagens.push(`${chave}: ${erro}`);
      erros++;
    }
  }
  const texto = erros
    ? `${feitos} baixado(s), ${erros} com erro.`
    : feitos ? `Pronto: ${feitos} ${feitos === 1 ? "publicação baixada" : "publicações baixadas"}.` : "Tudo em dia. Nada novo para baixar.";
  estado.fim = { texto, erros, mensagens };
  avisar(texto);
  mostrar();
});

const chaveDe = (p) => (p.edicao ? `${p.sigla}_${p.lingua}_${p.edicao}` : `${p.sigla}_${p.lingua}`);

async function procurarAtualizacoes(botao) {
  botao.disabled = true;
  botao.textContent = "Procurando…";
  try {
    const v = await invoke("verificar");
    if (v.novas.length) {
      const nomes = v.novas.map((n) => n.novo.publicacao).join(", ");
      avisar(`Versão nova: ${nomes}. Toque em Sincronizar agora para baixar.`);
    } else if (v.erros.length) {
      avisar(`Não consegui conferir: ${v.erros[0]}`);
    } else {
      avisar("Tudo em dia.");
    }
  } catch (e) { avisar(String(e)); }
  botao.disabled = false;
  botao.textContent = "Procurar atualizações";
}

// ---------- Baixados ----------

async function telaBaixados() {
  const [lista, espaco] = await Promise.all([invoke("baixados"), invoke("espaco_livre").catch(() => null)]);
  const tela = el("div", { class: "tela" });
  const ocupado = lista.reduce((s, b) => s + b.ocupado, 0);
  tela.append(el("div", { class: "disco" },
    el("div", {}, el("b", {}, String(lista.length)), el("span", {}, lista.length === 1 ? "publicação" : "publicações")),
    el("div", {}, el("b", {}, mb(ocupado)), el("span", {}, "ocupados na Biblioteca")),
    espaco && el("div", {}, el("b", {}, mb(espaco.livre)), el("span", {}, `livres no disco (de ${mb(espaco.total)})`))));

  if (!lista.length) {
    tela.append(el("div", { class: "vazio" }, el("h2", {}, "Nada baixado ainda"),
      el("p", {}, "Escolha publicações e toque em Sincronizar agora.")));
    return tela;
  }
  const filas = el("div", { class: "lista" });
  for (const b of lista) {
    const acoes = el("div", { class: "acoes" });
    if (estado.confirmando === b.chave) {
      acoes.append(el("span", { class: "confirmar" }, "Apagar da Biblioteca?"),
        el("button", { class: "botao perigo pequeno", onclick: () => apagar(b) }, "Apagar"),
        el("button", { class: "botao leve pequeno", onclick: () => { estado.confirmando = null; mostrar(); } }, "Cancelar"));
    } else {
      acoes.append(el("button", { class: "botao leve pequeno", onclick: () => { estado.confirmando = b.chave; mostrar(); } }, "Apagar"));
    }
    const titulo = b.titulo && b.titulo !== b.publicacao ? ` — ${b.titulo}` : "";
    filas.append(el("div", { class: "fila" }, capa(b.capa, b.pedido.sigla),
      el("div", { class: "corpo" },
        el("div", { class: "t" }, b.publicacao + titulo),
        el("div", { class: "m" }, `${b.chave} · ${nomeIdioma(b.pedido.lingua)} · ${mb(b.ocupado || b.tamanho)} · baixado em ${data(b.baixado_em)}`)),
      acoes));
  }
  tela.append(filas);
  return tela;
}

async function apagar(b) {
  try {
    await invoke("apagar", { chave: b.chave });
    avisar(`${b.publicacao} foi apagado.`);
  } catch (e) { avisar(String(e)); }
  estado.confirmando = null;
  mostrar();
}

// ---------- Abas ----------

const telas = { catalogo: telaCatalogo, selecao: telaSelecao, baixados: telaBaixados };

function trocarAba(aba) {
  estado.aba = aba;
  estado.confirmando = null;
  gravarPreferencia("aba", aba);
  mostrar();
}

let pedidoAtual = 0;
async function mostrar() {
  const meu = ++pedidoAtual;
  for (const b of document.querySelectorAll("#abas button")) b.classList.toggle("ativa", b.dataset.aba === estado.aba);
  caixaIdioma.style.visibility = estado.aba === "catalogo" ? "visible" : "hidden";
  try {
    const rolagem = conteudo.scrollTop;
    const tela = await telas[estado.aba]();
    if (meu !== pedidoAtual) return;
    conteudo.replaceChildren(tela);
    conteudo.scrollTop = rolagem;
  } catch (e) {
    if (meu !== pedidoAtual) return;
    conteudo.replaceChildren(el("div", { class: "vazio" }, el("h2", {}, "Algo deu errado"), el("p", {}, String(e))));
  }
}

for (const b of document.querySelectorAll("#abas button")) b.addEventListener("click", () => trocarAba(b.dataset.aba));

(async () => {
  // Os nomes dos idiomas servem às três abas.
  try { estado.idiomas = (await invoke("catalogo")).idiomas; preencherIdiomas(); } catch { /* a aba mostra o erro */ }
  const aba = new URLSearchParams(location.search).get("aba") || lerPreferencia("aba", "catalogo");
  estado.aba = telas[aba] ? aba : "catalogo";
  mostrar();
})();
