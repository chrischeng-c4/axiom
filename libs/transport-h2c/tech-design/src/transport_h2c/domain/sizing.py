from __future__ import annotations

import math


def recommended_connections(concurrency: int, parallelism: int) -> int:
    cap = max(parallelism, 1)
    if concurrency <= 2:
        return 1
    raw = math.ceil(math.log(concurrency))
    return min(max(raw, 1), cap)
