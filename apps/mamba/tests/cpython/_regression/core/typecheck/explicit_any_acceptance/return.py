# RUN: typecheck

from typing import Any


def collect() -> Any:
    return []


result = collect()
result.append(1)
result.append("two")
assert result == [1, "two"]
print("return:1:two")
