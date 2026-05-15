# ADR 0006: Natural Language Understanding and Intent Mapping

## Status
Proposed

## Context
JARVIS OS is a voice-first operating system. Current interaction models are limited to simple keyword matching. To achieve a truly intelligent system as depicted in Iron Man comics, we need a robust Natural Language Understanding (NLU) layer that can map voice input to system intents.

## Decision
We will implement an Intent-Based Architecture for the Voice Shell.
1.  **Intent Registry:** A central registry of system-wide intents (e.g., `GetSystemStatus`, `ControlEnergyFlow`, `InitiateDiagnostics`).
2.  **Entity Extraction:** The STT pipeline will be extended to extract entities (e.g., "Check status of *Core Reactor*").
3.  **Contextual Awareness:** The AI layer will maintain a conversation context to resolve pronouns and implicit references.

## Consequences
- Increased complexity in the AI module.
- Higher memory requirements for intent mapping tables.
- Improved user experience and alignment with the JARVIS vision.
