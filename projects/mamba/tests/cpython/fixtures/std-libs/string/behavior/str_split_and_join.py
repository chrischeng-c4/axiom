# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_split_and_join"
# subject = "str.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.split: str.split on whitespace and on a delimiter, plus the inverse str.join: 'hello world foo'.split()==['hello','world','foo'], ','.join(...) round-trips"""
import builtins  # noqa: F401

assert "hello world foo".split() == ["hello", "world", "foo"], "split on whitespace"
assert "a,b,c".split(",") == ["a", "b", "c"], "split on comma"
assert "a,,b".split(",") == ["a", "", "b"], "split keeps empty field"
assert ",".join(["a", "b", "c"]) == "a,b,c", "join with comma"
assert " ".join(["hello", "world"]) == "hello world", "join with space"
assert "".join(["a", "b", "c"]) == "abc", "join with empty"
print("str_split_and_join OK")
