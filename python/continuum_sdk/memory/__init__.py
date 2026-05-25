"""Continuum SDK Memory Module

Tiered memory system for conversation context management.

Features:
    - Short-term memory (recent messages)
    - Long-term memory (important facts)
    - Episodic memory (session history)
    - Semantic memory (knowledge base)
    - Automatic tier promotion

Tier Levels:
    - Tier 0: Immediate context (last N messages)
    - Tier 1: Short-term (active session)
    - Tier 2: Long-term (persisted facts)
    - Tier 3: Archive (historical data)

Quick Start:
    >>> from continuum_sdk.memory import Memory, MemoryTier
    >>>
    >>> memory = Memory()
    >>>
    >>> # Add entry
    >>> memory.add("User prefers Python", tier=MemoryTier.LONG_TERM)
    >>>
    >>> # Retrieve relevant context
    >>> context = memory.get_context("What language?")
    >>> print(context)  # "User prefers Python"
"""

from .layers import Memory, MemoryTier, MemoryEntry, TierProxy

__all__ = [
    "Memory",
    "MemoryTier",
    "MemoryEntry",
    "TierProxy",
]
