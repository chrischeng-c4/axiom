# RUN: typecheck
# EXPECT-ERROR: cannot infer return type for function `collect`

from typing import Any


def collect():
    return []


result = collect()
result.append(1)
result.append("two")
assert result == [1, "two"]
print("return:1:two")
