# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "attrgetter_is_callable"
# subject = "operator.attrgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter_is_callable (surface)."""
import operator

assert callable(operator.attrgetter)
print("attrgetter_is_callable OK")
