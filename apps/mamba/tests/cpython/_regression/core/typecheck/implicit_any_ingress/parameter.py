# RUN: typecheck
# EXPECT-ERROR: cannot infer type for parameter `items`

from typing import Any


def collect(items = []):
    items.append(1)
    items.append("two")
    return items


result = collect()
assert result == [1, "two"]
print("parameter:1:two")
