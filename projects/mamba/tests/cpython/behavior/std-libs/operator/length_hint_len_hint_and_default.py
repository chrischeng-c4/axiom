# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "length_hint_len_hint_and_default"
# subject = "operator.length_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: length_hint prefers __len__, falls back to __length_hint__, and returns the supplied default only when no length information is available"""
import operator

class Hinted:
    def __init__(self, value):
        self.value = value

    def __length_hint__(self):
        return self.value


assert operator.length_hint([], 2) == 0, "len wins over default"
assert operator.length_hint(iter([1, 2, 3])) == 3, "iterator length hint"
assert operator.length_hint(Hinted(5)) == 5, "explicit __length_hint__"
assert operator.length_hint(object(), 10) == 10, "fallback default"

print("length_hint_len_hint_and_default OK")
