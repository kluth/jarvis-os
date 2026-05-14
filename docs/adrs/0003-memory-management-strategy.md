# ADR 0003: Speicherverwaltungs-Strategie (Memory Management)

## Status
Vorgeschlagen

## Kontext
Der Kernel benötigt eine effiziente Verwaltung des physischen und virtuellen Speichers. Da x86_64 standardmäßig 4-Level-Paging verwendet, müssen wir entscheiden, wie wir den physischen Speicher im virtuellen Adressraum sichtbar machen und wie wir dynamischen Speicher (Heap) verwalten.

## Entscheidung
1.  **Physisches Memory Mapping:** Wir verwenden das **Complete Physical Memory Mapping**. Dabei wird der gesamte physische Speicher linear in einen hohen Bereich des virtuellen Adressraums gemappt (z.B. ab Offset `0x0000_4000_0000_0000`). Dies ermöglicht dem Kernel den Zugriff auf jede physische Adresse durch einfache Addition des Offsets.
2.  **Frame Allocator:** Für die Verwaltung physischer Speicherseiten (Frames) implementieren wir initial einen einfachen **Bump Allocator**, der auf der Memory-Map des Bootloaders basiert. Später wird dieser durch einen **Bitmap Allocator** oder **Buddy Allocator** ersetzt, um Speicherfreigaben zu unterstützen.
3.  **Page Table Management:** Wir nutzen die `x86_64` Crate-Abstraktionen (`OffsetPageTable`), um die 4-stufigen Seitentabellen sicher zu manipulieren.
4.  **Heap Allocator:** Wir integrieren die `linked_list_allocator` Crate für die erste Implementierung des Kern-Heaps. Dies ermöglicht die Nutzung von `alloc` (Box, Vec, etc.) im Kernel.

## Bounded Context
**MemoryContext**: Verantwortlich für die Zuweisung von physischen Frames, das Mapping von virtuellen Pages und die Verwaltung des Kernel-Heaps.

## Design Patterns
*   **Resource Pool:** Verwaltung der physischen Frames.
*   **Singleton/Provider:** Globaler Zugriff auf den Allocator.

## Konsequenzen
*   **Vorteile:** Schneller Zugriff auf physischen Speicher. Ermöglicht komplexe Datenstrukturen durch `alloc`.
*   **Nachteile:** Hoher virtueller Speicherverbrauch für das Mapping (bei x86_64 unproblematisch). Initialer Bump-Allocator unterstützt keine Deallokation von Frames.
