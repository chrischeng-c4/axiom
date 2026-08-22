# RUN: typecheck
# EXPECT-ERROR: cannot infer expression join type for binding `items`

from typing import Any


def exercise_expression_join(flag: bool) -> None:
    items = [] if flag else []
    items.append(1)
    items.append("two")
    assert items == [1, "two"]


exercise_expression_join(True)
print("expression_join:1:two")
