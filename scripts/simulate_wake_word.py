#!/usr/bin/env python3
"""
Simulate JARVIS Wake Word detection for Phoenix OS.
This script demonstrates how the system reacts to the "Phoenix" trigger.
"""

import time

def simulate_event(input_str):
    print(f"\n[USER]: {input_str}")
    time.sleep(0.5)

    normalized = input_str.lower()
    is_activated = normalized.startswith("phoenix")

    if is_activated:
        print("[KERNEL]: [Hermes] Wake word 'Phoenix' detected. Priority escalated to 1.0.")
        print("[AUDIO]:  [Aether] Playing: Wake Word Activated (Bright dual-tone chirp)")
        print("[JARVIS]: Speaking (Prosody: Calm): \"I am listening, user.\"")

        # Determine follow-up action
        if "fix" in normalized or "repair" in normalized:
            print("[KERNEL]: [Hermes] Dispatching action: SelfRepair")
            print("[KERNEL]: [Ghost Shell] Self-healing sequence initiated...")
        elif "research" in normalized:
            print("[KERNEL]: [Hermes] Dispatching action: KnowledgeQuery")
            print("[JARVIS]: [Synapse] Dispatching (NaturalLanguage): Hermes -> CognitiveCore (KnowledgeQuery)")
    else:
        print("[KERNEL]: [Hermes] Parsing intent: Low priority interaction (0.5)")
        print("[JARVIS]: [Synapse] Dispatching (NaturalLanguage): Hermes -> CognitiveCore (GeneralInteraction)")

if __name__ == "__main__":
    print("--- Phoenix OS Wake Word Simulation ---")
    simulate_event("Phoenix, research solid-state batteries.")
    time.sleep(1)
    simulate_event("Phoenix, fix the display driver.")
    time.sleep(1)
    simulate_event("What is the time?")
    print("\n--- Simulation Complete ---")
