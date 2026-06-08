# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "bare_special_form_repr_stable"
# subject = "typing.Any"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Any: the bare special forms have a stable, module-qualified repr: repr(typing.Any)=='typing.Any', repr(typing.NoReturn)=='typing.NoReturn', repr(typing.Never)=='typing.Never'"""
import typing

# repr of the bare special forms is stable and module-qualified.
assert repr(typing.Any) == "typing.Any"
assert repr(typing.NoReturn) == "typing.NoReturn"
assert repr(typing.Never) == "typing.Never"

print("bare_special_form_repr_stable OK")
