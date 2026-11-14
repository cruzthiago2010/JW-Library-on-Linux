#!/usr/bin/env python3
"""Gera ui/estilo/utilidades.css: as classes utilitárias (du-*) que o HTML das
publicações usa, com valores escritos por nós (paleta e espaçamentos próprios).

Não copia CSS de ninguém: só dá sentido às classes que aparecem no HTML.
Uso: python3 ferramentas/gerar_css.py > ui/estilo/utilidades.css
"""
import colorsys

UNIDADE = 4  # px por passo de espaçamento
PASSOS = list(range(0, 25))

# Matiz (graus) e saturação aproximados de cada família de cor.
FAMILIAS = {
    "gray": (220, 0.06), "coolGray": (215, 0.16), "warmGray": (30, 0.10),
    "orange": (24, 0.80), "gold": (42, 0.75), "amber": (38, 0.90), "yellow": (50, 0.90),
    "brown": (25, 0.35), "maroon": (350, 0.55), "red": (2, 0.70), "coral": (10, 0.75),
    "sage": (95, 0.18), "lime": (80, 0.55), "green": (130, 0.45), "mint": (160, 0.45),
    "teal": (180, 0.55), "turquoise": (175, 0.60), "cyan": (190, 0.65),
    "blue": (212, 0.70), "indigo": (232, 0.50), "violet": (262, 0.50), "purple": (275, 0.45),
}
# Claridade de cada tom (50 = quase branco, 900 = bem escuro).
CLARIDADE = {50: 0.96, 100: 0.92, 200: 0.85, 300: 0.75, 400: 0.64, 500: 0.54,
             600: 0.46, 700: 0.38, 800: 0.30, 900: 0.22}


def cor(familia, tom):
    h, s = FAMILIAS[familia]
    r, g, b = colorsys.hls_to_rgb(h / 360, CLARIDADE[tom], s)
    return "#%02x%02x%02x" % (round(r * 255), round(g * 255), round(b * 255))


def cores():
    for familia in FAMILIAS:
        for tom in CLARIDADE:
            yield f"{familia}-{tom}", cor(familia, tom)
            if familia[0].islower():
                yield f"{familia[0].upper()}{familia[1:]}-{tom}", cor(familia, tom)
    for nome, valor in {
        "white": "#ffffff", "black": "#000000",
        "textDefault": "var(--texto)", "textSubdued": "var(--texto-suave)",
        "borderDefault": "var(--borda)", "bgSecondary": "var(--fundo-bloco)",
    }.items():
        yield nome, valor


LADOS = {
    "": ["{p}"],
    "-top": ["{p}-top"], "-bottom": ["{p}-bottom"],
    "-inlineStart": ["{p}-inline-start"], "-inlineEnd": ["{p}-inline-end"],
    "-vertical": ["{p}-block"], "-horizontal": ["{p}-inline"],
}
VARIANTES = {"": None, "-desktopOnly": "(min-width: 601px)", "-mobileOnly": "(max-width: 600px)"}


def regra(seletor, corpo, midia=None):
    linha = f"{seletor}{{{corpo}}}"
    return f"@media {midia}{{{linha}}}" if midia else linha


def main():
    saida = ["/* Gerado por ferramentas/gerar_css.py — não editar à mão. */"]
    for prop, css in (("margin", "margin"), ("padding", "padding")):
        for lado, modelos in LADOS.items():
            for var, midia in VARIANTES.items():
                for n in PASSOS:
                    corpo = ";".join(f"{m.format(p=css)}:{n * UNIDADE}px" for m in modelos)
                    saida.append(regra(f".du-{prop}{lado}{var}--{n}", corpo, midia))
    # Espaço entre os filhos de um bloco.
    for lado, alvo in (("", "margin-block"), ("-vertical", "margin-block"),
                       ("-top", "margin-top"), ("-bottom", "margin-bottom")):
        for var, midia in VARIANTES.items():
            for n in PASSOS:
                saida.append(regra(f".du-margin-children{lado}{var}--{n}>*", f"{alvo}:{n * UNIDADE}px", midia))
    for nome, valor in cores():
        saida.append(f".du-color--{nome}{{color:{valor}}}")
        saida.append(f".du-bgColor--{nome}{{background-color:{valor}}}")
        saida.append(f".du-borderColor--{nome}{{border-color:{valor}}}")
    for n in (1, 2, 3, 4, 6, 8, 10, 12):
        saida.append(f".du-borderWidth--{n}{{border-width:{n}px}}")
        saida.append(f".du-borderWidth-inlineStart--{n}{{border-inline-start-width:{n}px}}")
    for lado, css in (("", "border"), ("-top", "border-top"), ("-bottom", "border-bottom"),
                      ("-inlineStart", "border-inline-start"), ("-vertical", "border-block")):
        for estilo in ("solid", "dashed", "dotted"):
            saida.append(f".du-borderStyle{lado}--{estilo}{{{css}-style:{estilo}}}")
    tamanhos = {"baseMinus2": -2, "baseMinus1": -1, "base": 0}
    tamanhos.update({f"basePlus{i}": i for i in range(1, 7)})
    for nome, passo in tamanhos.items():
        valor = f"{1.125 ** passo:.3f}em"
        saida.append(f".du-fontSize--{nome}{{font-size:{valor}}}")
        saida.append(regra(f".du-fontSize-desktopOnly--{nome}", f"font-size:{valor}", "(min-width: 601px)"))
    fracoes = {"full": "100%", "oneHalf": "50%", "oneQuarter": "25%", "threeQuarters": "75%",
               "oneThird": "33.333%", "twoThirds": "66.667%", "twoFifths": "40%", "oneSixth": "16.667%",
               "oneFifth": "20%", "threeFifths": "60%"}
    for nome, valor in fracoes.items():
        saida.append(f".du-width--{nome}{{width:{valor}}}")
        saida.append(regra(f".du-width-desktopOnly--{nome}", f"width:{valor}", "(min-width: 601px)"))
        saida.append(f".dc-column-width--{nome}{{flex:0 0 {valor};max-width:{valor}}}")
    for var, midia in VARIANTES.items():
        saida.append(regra(f".du-float{var}--inlineStart", "float:inline-start;float:left", midia))
        saida.append(regra(f".du-float{var}--inlineEnd", "float:inline-end;float:right", midia))
    for valor in ("left", "center", "right", "start", "end"):
        saida.append(f".du-textAlign--{valor}{{text-align:{valor}}}")
    saida += [
        ".du-clear--both{clear:both}",
        ".du-clearfix::after{content:'';display:table;clear:both}",
        ".du-listStyleType--none{list-style:none;padding-inline-start:0}",
        ".du-listStyleType--source{list-style:none}",
        ".du-fontWeight--normal{font-weight:400}",
        ".du-fontWeight--bold{font-weight:700}",
        ".du-letterSpacing--1{letter-spacing:.06em}",
        ".du-sticky{position:sticky;top:0}",
        ".du-bgPageDefault{background:var(--fundo)}",
        ".du-bgColorOpacity--90{opacity:.95}",
    ]
    print("\n".join(saida))


if __name__ == "__main__":
    main()
