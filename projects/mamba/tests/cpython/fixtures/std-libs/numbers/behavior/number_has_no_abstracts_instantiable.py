# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "number_has_no_abstracts_instantiable"
# subject = "numbers.Number"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Number: Number declares no abstract methods, so it is concrete and Number() instantiates without raising"""
import numbers

# Number is the tower root and declares zero abstract methods, so unlike the
# four lower ABCs it is concrete and instantiates with no TypeError.
assert numbers.Number.__abstractmethods__ == frozenset(), numbers.Number.__abstractmethods__

instance = numbers.Number()
assert isinstance(instance, numbers.Number), type(instance)

print("number_has_no_abstracts_instantiable OK")
