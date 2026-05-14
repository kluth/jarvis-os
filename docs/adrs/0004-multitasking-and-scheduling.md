# ADR 0004: Multitasking und Scheduling-Strategie

## Status
Vorgeschlagen

## Kontext
JARVIS OS muss in der Lage sein, mehrere Aufgaben (Tasks) quasi-gleichzeitig auszuführen. Dies ist die Voraussetzung für Hintergrundprozesse wie die Hardware-Erkennung und die KI-Kommunikation. Wir müssen entscheiden, wie Tasks repräsentiert werden und welcher Scheduling-Algorithmus zum Einsatz kommt.

## Entscheidung
1.  **Kooperatives vs. Präemptives Multitasking:** Wir implementieren initial **kooperatives Multitasking** unter Nutzung von Rusts `async/await` Infrastruktur. Dies ermöglicht eine effiziente Verwaltung vieler Tasks ohne die Komplexität voller Thread-Kontext-Switches (präemptiv) in der frühen Phase.
2.  **Task Repräsentation:** Ein Task wird durch ein `Future` repräsentiert, das eine atomare Einheit von Arbeit beschreibt.
3.  **Executor:** Wir implementieren einen einfachen `Executor`, der eine Warteschlange von Tasks (Futures) verwaltet und diese abarbeitet (pollt), bis sie bereit sind.
4.  **Waker-Mechanismus:** Um CPU-Zyklen zu sparen, nutzen wir einen `Waker`-Mechanismus, der Tasks nur dann pollt, wenn ein Ereignis (z.B. ein Timer-Interrupt oder Daten auf dem Bus) eingetreten ist.
5.  **Vorbereitung auf Präemption:** Langfristig wird ein präemptiver Scheduler (Multilevel Feedback Queue) hinzugefügt, der auf Hardware-Timern (APIC) basiert.

## Design Patterns
*   **Strategy Pattern:** Abstraktion des Executors, um später zwischen kooperativem und präemptivem Scheduling wechseln zu können.
*   **State Pattern:** Verwaltung des Task-Status (Pending, Ready, Completed).

## Konsequenzen
*   **Vorteile:** Geringer Overhead, nutzt Rusts Typsicherheit für asynchrone Programmierung, ideal für I/O-lastige JARVIS-Module.
*   **Nachteile:** Tasks müssen aktiv `yield`en (bzw. `.await` nutzen), um andere Tasks nicht zu blockieren.
