# Phoenix OS System Architecture Specification

This document serves as the formal blueprint for the Phoenix OS and JARVIS platform.

## 1. Architectural Overview

Phoenix OS is an AI-Native operating system where the core kernel and services provide a robust, API-first platform for a modular AI Orchestration layer.

```text
                 User
                  │
        Interface (Voice/GUI/CLI)
                  │
           AI Orchestrator (JARVIS)
    ┌─────────────┼─────────────┐
    │             │             │
  Memory      Planning      Research
    │             │             │
  Skills      Automation      Coding
    │             │             │
 Knowledge    Security      Vision
                  │
             Service Bus (IPC)
                  │
         Phoenix OS Services
    ┌─────────────┼─────────────┐
    │             │             │
 Filesystem    Network       Security
                  │
        Kernel • Drivers • Memory
```

## 2. AI Orchestration Layer

### 2.1. AI Orchestrator
The central coordinator responsible for:
- **Executive Control**: Choosing goals, prioritizing tasks, and allocating system resources.
- **Agent Coordination**: Delegating tasks to specialized agents (Coding, Research, etc.).
- **Uncertainty Tracking**: Monitoring confidence levels and requesting user clarification.

### 2.2. Specialized Agents
- **Conversation Agent**: Manages natural language interaction and personality.
- **Planning Agent**: Breaks down high-level goals into actionable milestones and tasks.
- **Memory Agent**: Manages the hierarchical memory system.
- **Research Agent**: Gathers and synthesizes information from multiple sources with citation tracking.
- **Automation Agent**: Manages recurring workflows and system-level task scheduling.
- **Security Agent**: Audits all AI actions against the capability-based security model.

### 2.3. Executive Controller & Attention Manager
- **Goal Management**: Maintains a hierarchy of Goal -> Milestone -> Task -> Action.
- **Resource Allocation**: Adjusts model selection and agent activity based on hardware constraints (CPU, RAM, Battery).
- **Attention Management**: Limits information processing per module to maintain system responsiveness.

## 3. Hierarchical Memory System

Instead of a single store, the Memory Agent manages specialized layers:
1. **Working Memory**: Current conversation and immediate task context.
2. **Episodic Memory**: Recording of important system and user events.
3. **Semantic Memory**: General facts, concepts, and relationships (Knowledge Graph).
4. **Procedural Memory**: "How-to" knowledge for performing tasks (Skill Library).
5. **Preference Memory**: User-specific choices, tone preferences, and formality levels.
6. **Project Memory**: Context specific to active files, codebases, and documentation.
7. **Archive Memory**: Long-term storage with decay and duplicate detection.

## 4. Skill & Tool Ecosystem

### 4.1. Skill System
Skills are reusable, metadata-rich capabilities:
- **Metadata**: Required permissions, tools, models, estimated runtime, and success criteria.
- **Composition**: New skills can be created by combining existing ones.

### 4.2. Unified Tool Interface
Every system capability is a tool with a consistent interface:
- Web Browser, Compiler, Git, Databases, Smart Home, etc.
- JARVIS discovers and invokes tools through the Service Bus.

## 5. Security & Governance

### 5.1. Capability-Based Security
- **Least-Privilege**: Every agent and plugin runs with the minimum required permissions.
- **Audit Logs**: Every privileged action is recorded in a tamper-proof audit log.
- **Human-in-the-Loop**: High-risk actions require explicit human approval.

### 5.2. Governance Policies
- **Autonomy levels**: User-defined boundaries for autonomous action.
- **Privacy**: Local-first learning; user-controlled memory editing and deletion.

## 6. System Services & IPC

### 6.1. Service Bus
- **Event-Driven**: Lightweight Pub/Sub system for system-wide events.
- **Structured Data**: Mandatory use of structured schemas (Binary/JSON/CBOR) for all communication.

### 6.2. Observability
- **Metrics & Tracing**: Real-time performance dashboards and event tracing.
- **Health Checks**: Continuous monitoring of OS and AI service health.
