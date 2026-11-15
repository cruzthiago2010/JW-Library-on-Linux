#!/bin/bash
# Abre o app numa tela virtual (Xvfb :77, sem mexer no monitor de verdade) e tira prints seguindo cliques. Uso: ferramentas/print.sh <pasta> nome:x,y[;x,y...] ...
# Cada argumento: nome do print e os cliques feitos antes dele (coordenadas da janela).
export DISPLAY=:77
pgrep -x Xvfb >/dev/null || { setsid -f Xvfb :77 -screen 0 1456x900x24 -nolisten tcp >/dev/null 2>&1; sleep 1; }
SAIDA="$1"; shift
B="${CARGO_TARGET_DIR:-$(dirname "$0")/../target}/debug/leitor-jw"
pkill -x leitor-jw; sleep 1
setsid -f "$B" > "$SAIDA/app.log" 2>&1
for i in $(seq 20); do W=$(xdotool search --name "^Leitor JW$" | tail -1); [ -n "$W" ] && break; sleep 0.5; done
[ -z "$W" ] && { echo "janela do app não apareceu"; exit 1; }
xdotool windowmove "$W" 0 0; xdotool windowsize "$W" 1456 860; xdotool windowfocus "$W" 2>/dev/null; sleep 3
for passo in "$@"; do
  nome="${passo%%:*}"; cliques="${passo#*:}"
  IFS=';' read -ra lista <<< "$cliques"
  for c in "${lista[@]}"; do
    [ -z "$c" ] && continue
    if [[ "$c" == arrastar* ]]; then IFS=, read -r x1 y1 x2 y2 <<< "${c#arrastar}"; xdotool mousemove --window "$W" "$x1" "$y1" mousedown 1; sleep 0.3; xdotool mousemove --window "$W" "$x2" "$y2"; sleep 0.3; xdotool mouseup 1
    elif [[ "$c" == escrever* ]]; then xdotool type --delay 30 "${c#escrever}"
    elif [[ "$c" == rolar* ]]; then n="${c#rolar}"; xdotool mousemove --window "$W" 700 500; for k in $(seq "$n"); do xdotool click 5; done
    else xdotool mousemove --window "$W" "${c%,*}" "${c#*,}" click 1; fi
    sleep 1.5
  done
  import -window "$W" "$SAIDA/$nome.png"
done
