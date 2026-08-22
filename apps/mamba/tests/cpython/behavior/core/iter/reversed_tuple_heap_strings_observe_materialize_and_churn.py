# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "iter"
# dimension = "behavior"
# case = "reversed_tuple_heap_strings_observe_materialize_and_churn"
# subject = "builtins.reversed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
"""Observe reversed tuple contents before releasing the materialized list."""


def heap_text(label: str, number: int) -> str:
    return label + "-" + str(number) + "-runtime"


def make_source() -> tuple[str, str, str]:
    return (heap_text("left", 17), heap_text("middle", 29), heap_text("right", 41))


def materialize_and_observe(source: tuple[str, str, str]) -> None:
    materialized = list(reversed(source))
    assert len(materialized) == 3
    assert materialized == [
        "right-41-runtime",
        "middle-29-runtime",
        "left-17-runtime",
    ]
    return None


source = make_source()
materialize_and_observe(source)
churn = ["churn-" + str(i) + "-runtime" for i in range(16)]
assert len(churn) == 16
assert churn[0] == "churn-0-runtime"
assert churn[-1] == "churn-15-runtime"
assert source == ("left-17-runtime", "middle-29-runtime", "right-41-runtime")
print("reversed_tuple_heap_strings_observe_materialize_and_churn OK")
