# RUN: typecheck

from typing import Any


def collect(items: Any = []):
    items.append(1)
    items.append("two")
    return items


result = collect()
assert result == [1, "two"]
print("parameter:1:two")
