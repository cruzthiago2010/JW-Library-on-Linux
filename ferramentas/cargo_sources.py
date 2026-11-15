#!/usr/bin/env python3
"""Gera empacotar/flatpak/cargo-sources.json a partir do Cargo.lock: as crates do
crates.io como fontes do flatpak-builder (para compilar sem internet, como o
Flathub exige). Faz o mesmo que o flatpak-cargo-generator, só para crates.io.
Uso: python3 ferramentas/cargo_sources.py Cargo.lock > empacotar/flatpak/cargo-sources.json
"""
import json
import sys
import tomllib

lock = tomllib.load(open(sys.argv[1], "rb"))
fontes = []
for p in lock["package"]:
    if p.get("source", "").startswith("registry+https://github.com/rust-lang/crates.io-index") and p.get("checksum"):
        nome, versao, soma = p["name"], p["version"], p["checksum"]
        destino = f"cargo/vendor/{nome}-{versao}"
        fontes.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{nome}/{nome}-{versao}.crate",
            "sha256": soma,
            "dest": destino,
        })
        fontes.append({
            "type": "inline",
            "contents": json.dumps({"package": soma, "files": {}}),
            "dest": destino,
            "dest-filename": ".cargo-checksum.json",
        })
    elif "source" in p:
        sys.exit(f"fonte não suportada: {p['name']} {p['source']}")
fontes.append({
    "type": "inline",
    "contents": '[source.vendored-sources]\ndirectory = "cargo/vendor"\n\n[source.crates-io]\nreplace-with = "vendored-sources"\n',
    "dest": "cargo",
    "dest-filename": "config",
})
json.dump(fontes, sys.stdout, indent=2)
