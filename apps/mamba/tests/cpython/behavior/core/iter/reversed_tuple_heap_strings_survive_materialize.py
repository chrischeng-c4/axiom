# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "iter"
# dimension = "behavior"
# case = "reversed_tuple_heap_strings_survive_materialize"
# subject = "builtins.reversed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
"""Reversed tuple elements survive materializing and releasing the iterator."""


def heap_text(label: str, number: int) -> str:
    return label + "-" + str(number) + "-runtime"


def make_source() -> tuple[str, str, str]:
    return (heap_text("left", 17), heap_text("middle", 29), heap_text("right", 41))


def materialize_and_discard(source: tuple[str, str, str]) -> None:
    materialized = list(reversed(source))
    return None


source = make_source()
materialize_and_discard(source)
churn = ["churn-" + str(i) + "-runtime" for i in range(16)]
assert source == ("left-17-runtime", "middle-29-runtime", "right-41-runtime")
print("reversed_tuple_heap_strings_survive_materialize OK")
