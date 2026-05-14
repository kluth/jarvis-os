# JARVIS OS

Das allumfassende Betriebssystem-Projekt.

## CI/CD Status
Die Kompilierung findet automatisiert auf GitHub statt, um lokale Ressourcen zu schonen.

[![Build JARVIS OS](https://github.com/kluth/jarvis-os/actions/workflows/build.yml/badge.svg)](https://github.com/kluth/jarvis-os/actions/workflows/build.yml)

## Lokale Entwicklung
Da der Build-Prozess sehr ressourcenintensiv ist, wird empfohlen, die Artefakte aus den GitHub Actions herunterzuladen.

Zum Testen in QEMU (lokal):
1. Artefakt `jarvis-kernel` herunterladen.
2. `qemu-system-x86_64 -drive format=raw,file=target/x86_64-jarvis_os/debug/jarvis-kernel` (Pfad anpassen).

*Hinweis: Ein vollständiges Disk-Image-Tooling wird in Phase 1 noch finalisiert.*
