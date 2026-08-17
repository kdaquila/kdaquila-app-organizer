"""Tuning knobs for the auth feature.

No `def` and no `class`, so this file is free to be named after its topic
rather than after a single export -- and the line budget does not apply.
"""

type Seconds = int

TOKEN_TTL: Seconds = 900
REFRESH_TTL: Seconds = 86_400
MAX_ATTEMPTS = 5
LOCKOUT_TTL: Seconds = 300
BACKOFF_BASE = 2


def _jitter(attempt: int) -> float:
    return BACKOFF_BASE**attempt
