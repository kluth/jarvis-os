# ADR 0004: Multitasking and Scheduling Strategy

## Status
Proposed

## Context
JARVIS OS must be able to execute multiple tasks quasi-simultaneously. This is a prerequisite for background processes such as hardware detection and AI communication. We must decide how tasks are represented and which scheduling algorithm will be used.

## Decision
1.  **Cooperative vs. Preemptive Multitasking:** We initially implement **cooperative multitasking** using Rust's `async/await` infrastructure. This allows for efficient management of many tasks without the complexity of full thread context switches (preemptive) in the early phase.
2.  **Task Representation:** A task is represented by a `Future` that describes an atomic unit of work.
3.  **Executor:** We implement a simple `Executor` that manages a queue of tasks (futures) and processes (polls) them until they are ready.
4.  **Waker Mechanism:** To save CPU cycles, we use a `Waker` mechanism that polls tasks only when an event (e.g., a timer interrupt or data on the bus) has occurred.
5.  **Preparation for Preemption:** In the long term, a preemptive scheduler (Multilevel Feedback Queue) based on hardware timers (APIC) will be added.

## Design Patterns
*   **Strategy Pattern:** Abstraction of the executor to allow switching between cooperative and preemptive scheduling later.
*   **State Pattern:** Management of task status (Pending, Ready, Completed).

## Consequences
*   **Advantages:** Low overhead, utilizes Rust's type safety for asynchronous programming, ideal for I/O-heavy JARVIS modules.
*   **Disadvantages:** Tasks must actively `yield` (or use `.await`) to avoid blocking other tasks.
