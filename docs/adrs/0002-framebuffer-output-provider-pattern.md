# ADR 0002: Framebuffer Output und Provider Pattern

## Status
Vorgeschlagen

## Kontext
Der Kernel benötigt eine Möglichkeit, Informationen visuell auszugeben. Da wir UEFI verwenden, stellt uns der Bootloader einen Framebuffer (Linear Buffer) zur Verfügung. Wir müssen entscheiden, wie wir den Zugriff auf diesen Framebuffer strukturieren, um Thread-Sicherheit (auch vor Einführung von Scheduler/Mutexes) und Abstraktion zu gewährleisten.

## Entscheidung
1.  **Framebuffer-Abstraktion:** Wir implementieren einen `Writer`, der auf dem vom Bootloader bereitgestellten Framebuffer operiert.
2.  **Provider Pattern:** Wir nutzen das Provider Pattern, um eine globale Instanz des Writers bereitzustellen. Da wir noch keine Mutexes haben, nutzen wir initial eine `Locked` Abstraktion (z.B. ein einfacher Spinlock oder `lazy_static` mit Spinlock, falls verfügbar, sonst eine eigene minimalistische Implementierung).
3.  **Schriftart:** Für die erste Version nutzen wir eine einfache, eingebettete Bitmap-Schriftart (z.B. 8x16), um Text auf den Framebuffer zu zeichnen.
4.  **Scrolling:** Wir implementieren einen linearen Puffer-Scroll-Algorithmus, der den Speicherinhalt nach oben verschiebt, wenn der Bildschirm voll ist.

## Bounded Context
**DisplayContext**: Verantwortlich für die Ansteuerung der Grafik-Hardware (Framebuffer) und die Darstellung von Glyphen.

## Design Patterns
*   **Provider Pattern:** Globaler Zugriff auf den Display-Treiber.
*   **Strategy Pattern (Vorbereitung):** Abstraktion der Zeichenoperationen, um später zwischen verschiedenen Grafikmodi oder Architekturen wechseln zu können.

## Konsequenzen
*   **Vorteile:** Frühes visuelles Feedback für Debugging. Klare Trennung zwischen Text-Logik und Pixel-Logik.
*   **Nachteile:** Zusätzlicher Overhead für das Zeichnen von Pixeln im Vergleich zum VGA-Textmodus (der in UEFI nicht garantiert ist).
