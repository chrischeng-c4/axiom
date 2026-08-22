# RUN: typecheck
# EXPECT-ERROR: cannot infer type for global binding `items`

from typing import Any


items = []
items.append(1)
items.append("two")
assert items == [1, "two"]


print("global_binding:1:two")
