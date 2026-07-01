# JARVIS Cognitive Architecture

This document defines the high-level cognitive architecture for JARVIS, the orchestration layer of Phoenix OS.

## 1. Modular Brain (Cognitive Architecture)

JARVIS is composed of specialized, independent modules:

- **Personality**: Manages interaction style and tone.
- **Memory**: Handles short-term and long-term storage/retrieval.
- **Planner**: Breaks down goals into actionable tasks.
- **Reasoning**: Performs logical analysis and decision making.
- **Research**: Gathers information from local and remote sources.
- **Vision**: Processes visual data (images, screenshots, camera).
- **Voice**: Handles STT (Speech-to-Text) and TTS (Text-to-Speech).
- **Coding**: Specialized in software development and debugging.
- **Automation**: Executes recurring workflows and scripts.
- **Security**: Audits actions against the capability-based security model.
- **Learning**: Extracts facts and preferences from interactions.

## 2. Multi-Agent System

JARVIS utilizes a "Commander-Agent" model:

- **Commander**: The central coordinator that interacts with the user.
- **Specialized Agents**: Coding, Research, Security, Design, Automation, Memory, and Diagnostics agents.
- **Coordination**: The Commander delegates tasks to specialized agents rather than performing all work itself.

## 3. AI Message Bus

Communication between JARVIS modules and agents is handled via an **Event Bus** rather than direct coupling. This allows for:

- Decoupling of components.
- Easier integration of new modules.
- Asynchronous processing of complex tasks.

## 4. Context Engine

A centralized engine that tracks:
- Current project and goals.
- Active files and running applications.
- User intent and conversation history.
- Available tools and recent work.

## 5. Long-Term Memory Layers

Memory is organized into hierarchical layers for efficient retrieval:
1. **Working Memory**: Last few minutes of interaction.
2. **Session Memory**: Current task context.
3. **Project Memory**: Knowledge specific to the current project.
4. **Long-Term Memory**: User history and established preferences.
5. **Knowledge Database**: General research and facts.
6. **Skill Database**: Learned workflows and automation routines.

## 6. Skill & Plugin System

- **Skills**: Reusable compositions of basic actions (e.g., "Create Website" = Research + Plan + Code + Test).
- **Plugins**: A standard for third-party extensions (Skills, Agents, Models, Drivers, Themes).

## 7. Intelligent Resource Management

Optimized for low-end hardware:
- **Model Router**: Switches between small local models (for simple tasks) and cloud models (for complex reasoning).
- **Dynamic Loading**: Unloads idle models and suspends inactive agents.
- **Compression**: Uses memory compression (zram) for model weights and context.

## 8. Safety & Auditing

- **Capability-Based Security**: JARVIS must have a specific "capability token" for every sensitive action.
- **Versioned Memory**: Knowledge evolution is tracked, allowing for rollbacks and auditing.
- **Transparency**: Reasoning can be explained upon request.
