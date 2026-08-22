# RUN: typecheck
# EXPECT-ERROR: cannot infer type for class attribute `items`

from typing import Any


class Bucket:
    items = []


Bucket.items.append(1)
Bucket.items.append("two")
assert Bucket.items == [1, "two"]
print("class_attribute:1:two")
