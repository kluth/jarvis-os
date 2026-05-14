# ADR 0001: Bootloader und Rust no_std Umgebung

## Status
Akzeptiert

## Kontext
Für die Entwicklung von JARVIS OS "von Grund auf" benötigen wir eine Umgebung, die ohne ein bestehendes Betriebssystem (Bare Metal) lauffähig ist. Die Zielarchitektur ist initial x86_64, da diese eine breite Hardware-Basis bietet und in QEMU exzellent emuliert werden kann. Wir müssen entscheiden, wie das System startet (Bootloader) und wie wir Rust in dieser Umgebung konfigurieren.

## Entscheidung
1.  **Programmiersprache:** Wir verwenden **Rust** mit dem Attribut `#![no_std]`, um die Standardbibliothek (die ein OS voraussetzt) auszuschließen. Wir nutzen die `core`-Library.
2.  **Bootloader:** Wir verwenden **UEFI (Unified Extensible Firmware Interface)** anstelle des veralteten BIOS. UEFI bietet moderne Features wie Speicher-Mapping und Grafik-Initialisierung (GOP) bereits vor dem Kernel-Start.
3.  **Bootloader-Implementierung:** Wir nutzen das `bootloader` Crate (v0.11 oder höher), da es eine nahtlose Integration in den Rust-Build-Prozess bietet und den Kernel direkt in den 64-Bit Long Mode versetzt.
4.  **Architektur-Muster:**
    *   **Abstract Factory:** Wir definieren Traits für grundlegende Hardware-Interaktionen (z.B. `Writer`, `Hal`), um den Kernel später leicht auf andere Architekturen (ARM, RISC-V) portieren zu können.
    *   **Chain of Responsibility:** Der Boot-Prozess wird in klar definierte Stufen unterteilt (UEFI -> Bootloader -> Kernel Init -> Module Init).

## Konsequenzen
*   **Vorteile:** 
    *   Volle Kontrolle über die Hardware von der ersten Instruktion an.
    *   Typsicherheit und Speichersicherheit durch Rust auch im Kernel.
    *   Zukunftssicheres Boot-Verfahren durch UEFI.
*   **Nachteile:**
    *   Hoher initialer Aufwand für die Einrichtung (Cross-Compilation, JSON-Targets).
    *   Keine Nutzung von Standard-Rust-Features wie `std::vec` oder `std::string` ohne eigenen Allocator.

## Verifizierung
*   Kompilierung mit `cargo build --target x86_64-unknown-none`.
*   Erfolgreicher Boot-Vorgang in **QEMU**, erkennbar an einem Framebuffer-Output.
