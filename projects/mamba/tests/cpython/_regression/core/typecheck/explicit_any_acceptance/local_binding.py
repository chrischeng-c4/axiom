# RUN: typecheck

from typing import Any


def exercise_local_binding() -> None:
    items: Any = []
    items.append(1)
    items.append("two")
    assert items == [1, "two"]


exercise_local_binding()
print("local_binding:1:two")
