# JARVIS Cognitive Architecture

This document defines the cognitive model for JARVIS, integrated into the Phoenix OS AI Orchestration layer.

## 1. The AI Orchestrator Model

JARVIS acts as the **AI Orchestrator**, a central nervous system that coordinates specialized agents and manages the executive functions of the OS.

### 1.1. Core Components
- **Executive Controller**: The "Prefrontal Cortex" of JARVIS. It chooses goals, pauses work when priorities shift, and manages the lifecycle of all agents.
- **Attention Manager**: Filters and prioritizes inputs to ensure the system remains responsive on low-end hardware.
- **Reflection Module**: An introspective layer that reviews completed tasks to identify errors and propose workflow improvements.

## 2. Multi-Agent Coordination

Instead of a monolithic AI, JARVIS coordinates a swarm of specialized agents via the **AI Message Bus**:

- **Commander (Personality Layer)**: The user-facing identity. Calm, helpful, and professional.
- **Specialized Agents**:
    - **Coding Agent**: Handles development and debugging.
    - **Research Agent**: Multi-source info gathering with fact-checking.
    - **Planning Agent**: Hierarchical goal decomposition.
    - **Memory Agent**: Hierarchical storage and retrieval.
    - **Security Agent**: Capability auditing and permission management.

## 3. Dynamic Lexicon & Learning

JARVIS maintains a multi-layered vocabulary system that ensures it can understand and adapt to user intent:

- **The Lexicon Registry**: A primary mapping of keywords and synonyms to system intents (e.g., "fix", "repair", "heal" all mapping to `SelfRepair`).
- **Semantic Fallback**: If a command keyword is not found, JARVIS queries the **Mnemosyne Knowledge Graph** for existing concept nodes to infer context.
- **The Learning Loop**: Completely unknown terms trigger a clarify-and-index sequence. JARVIS will ask for clarification via the Harmonic UI and then create a new permanent node in Mnemosyne.

## 4. Memory Architecture

Memory is implemented as a multi-layered system managed by the **Memory Agent**:

- **Short-Term (Working/Session)**: Immediate context.
- **Mid-Term (Project/Episodic)**: Task and event history.
- **Long-Term (Semantic/Preference)**: Fact-base and user profile.
- **Procedural (Skill Library)**: Reusable automation routines and workflows.

## 5. Interaction Modes

JARVIS supports multimodal interaction:
- **Voice**: Conversational STT/TTS.
- **Text**: Natural language shell.
- **Visual**: Vision engine for UI understanding and camera data.
- **Symbolic**: Direct API/Structured data interaction with OS services.

## 6. Reasoning & Uncertainty

JARVIS tracks **Confidence Levels** for every action. If confidence is below a threshold, the system is required to:
1. Record the uncertainty.
2. Formulate a clarifying question for the user.
3. Pause execution until the ambiguity is resolved.
