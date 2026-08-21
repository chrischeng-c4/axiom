# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "get_annotations_raw_and_eval_str"
# subject = "inspect.get_annotations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.get_annotations: get_annotations() returns raw annotations by default and resolves stringized ones only with eval_str=True; an unannotated object yields {}"""
import inspect

def fn(a: int, b: str) -> bool:
    return True

assert inspect.get_annotations(fn) == {"a": int, "b": str, "return": bool}, "raw annos"

fn.__annotations__ = {"a": "int", "b": "str"}
assert inspect.get_annotations(fn) == {"a": "int", "b": "str"}, "stringized raw"
assert inspect.get_annotations(fn, eval_str=True) == {"a": int, "b": str}, "stringized eval"

def plain(x):
    return x

assert inspect.get_annotations(plain) == {}, "no annotations"

print("get_annotations_raw_and_eval_str OK")
