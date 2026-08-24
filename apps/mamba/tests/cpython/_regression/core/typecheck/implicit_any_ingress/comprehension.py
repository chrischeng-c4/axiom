# RUN: typecheck
# EXPECT-ERROR: cannot infer comprehension type for binding `items`

from typing import Any


def exercise_comprehension() -> None:
    items = [item for item in []]
    items.append(1)
    items.append("two")
    assert items == [1, "two"]


exercise_comprehension()
print("comprehension:1:two")
