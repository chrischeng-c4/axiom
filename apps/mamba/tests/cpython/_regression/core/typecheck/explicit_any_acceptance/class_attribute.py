# RUN: typecheck

from typing import Any


class Bucket:
    items: Any = []


Bucket.items.append(1)
Bucket.items.append("two")
assert Bucket.items == [1, "two"]
print("class_attribute:1:two")
