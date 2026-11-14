// Ícones próprios do Leitor JW (traços simples, 24×24).
const ICONES = {
  menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
  voltar: '<path d="M19 12H5M11 6l-6 6 6 6"/>',
  inicio: '<path d="M4 11l8-7 8 7v9h-5v-6h-6v6H4z"/>',
  biblia: '<path d="M12 6c-2-1.5-5-2-8-1.5v14c3-.5 6 0 8 1.5 2-1.5 5-2 8-1.5v-14c-3-.5-6 0-8 1.5zM12 6v14"/>',
  biblioteca: '<path d="M4 5h5v15H4zM10 5h4v15h-4zM15.5 5.5l4-1 2.5 14.5-4 1z"/>',
  reunioes: '<circle cx="12" cy="6" r="2.5"/><path d="M8 13c0-2.5 1.8-4 4-4s4 1.5 4 4M5 14h14l-1.5 6h-11z"/>',
  estudo: '<path d="M7 4h10l4 6-9 10L3 10z M3 10h18M9 4l3 16 3-16"/>',
  importar: '<path d="M12 4v11M7 10l5 5 5-5M5 19h14"/>',
  mais: '<circle cx="5" cy="12" r="1.3"/><circle cx="12" cy="12" r="1.3"/><circle cx="19" cy="12" r="1.3"/>',
  esq: '<path d="M15 5l-7 7 7 7"/>',
  dir: '<path d="M9 5l7 7-7 7"/>',
  painel: '<path d="M7 4h10l4 6-9 10L3 10z M3 10h18"/>',
  fechar_painel: '<path d="M5 12h11M11 6l6 6-6 6M20 5v14"/>',
  historico: '<path d="M4 12a8 8 0 1 0 2.4-5.7M4 4v4h4M12 8v4l3 2"/>',
  livro: '<path d="M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3zM5 17a3 3 0 0 1 3-3h11"/>',
  revista: '<rect x="5" y="3" width="14" height="18" rx="1"/><path d="M8 7h8M8 11h8M8 15h5"/>',
  brochura: '<path d="M6 3h9l3 3v15H6zM15 3v3h3M9 10h6M9 14h6"/>',
  folheto: '<path d="M4 6l5-2 6 2 5-2v14l-5 2-6-2-5 2zM9 4v14M15 6v14"/>',
  apostila: '<rect x="4" y="4" width="16" height="16" rx="1"/><path d="M8 4v16M11 9h6M11 13h6"/>',
  ordenar: '<path d="M7 4v16M4 17l3 3 3-3M17 20V4M14 7l3-3 3 3"/>',
  buscar: '<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>',
  texto: '<path d="M4 19l5-14 5 14M6 14h6M15 19l3-8 3 8M16 17h4"/>',
  copiar: '<rect x="8" y="8" width="12" height="12" rx="1.5"/><path d="M16 8V5.5A1.5 1.5 0 0 0 14.5 4h-9A1.5 1.5 0 0 0 4 5.5v9A1.5 1.5 0 0 0 5.5 16H8"/>',
  nota: '<path d="M5 4h10l4 4v12H5z"/><path d="M15 4v4h4M9 13h6M12 10v6"/>',
  lixeira: '<path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v6M14 11v6"/>',
  marcador: '<path d="M7 3h10v18l-5-4-5 4z"/>',
  nuvem: '<path d="M7 18h10a4 4 0 0 0 .5-7.97A6 6 0 0 0 6.1 9.1 4.5 4.5 0 0 0 7 18z"/><path d="M12 16v-6M9.5 12.5L12 10l2.5 2.5"/>',
  config: '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1"/>',
};
function icone(nome, classe = 'ic') {
  return `<svg class="${classe}" viewBox="0 0 24 24" aria-hidden="true">${ICONES[nome] || ''}</svg>`;
}
