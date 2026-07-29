# RUN: typecheck

from typing import Any


items: Any = []
items.append(1)
items.append("two")
assert items == [1, "two"]


print("global_binding:1:two")
