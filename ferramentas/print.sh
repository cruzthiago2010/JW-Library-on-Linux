#!/bin/bash
# Abre o app no monitor (:0) e tira prints seguindo cliques. Uso: ferramentas/print.sh <pasta> nome:x,y[;x,y...] ...
# Cada argumento: nome do print e os cliques feitos antes dele (coordenadas da janela).
export DISPLAY=:0
SAIDA="$1"; shift
B="$HOME/Documents/Aplicativo Thiago/nativo/target/debug/leitor-jw"
pkill -x leitor-jw; sleep 1
setsid -f "$B" > "$SAIDA/app.log" 2>&1
for i in $(seq 20); do W=$(xdotool search --name "^Leitor JW$" | tail -1); [ -n "$W" ] && break; sleep 0.5; done
xdotool windowactivate "$W"; xdotool windowmove "$W" 0 40; xdotool windowsize "$W" 1456 860; sleep 3
for passo in "$@"; do
  nome="${passo%%:*}"; cliques="${passo#*:}"
  IFS=';' read -ra lista <<< "$cliques"
  for c in "${lista[@]}"; do
    [ -z "$c" ] && continue
    if [[ "$c" == rolar* ]]; then n="${c#rolar}"; xdotool mousemove --window "$W" 700 500; for k in $(seq "$n"); do xdotool click 5; done
    else xdotool mousemove --window "$W" "${c%,*}" "${c#*,}" click 1; fi
    sleep 1.5
  done
  import -window "$W" "$SAIDA/$nome.png"
done
