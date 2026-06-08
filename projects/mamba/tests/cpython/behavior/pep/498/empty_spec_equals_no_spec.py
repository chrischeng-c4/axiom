# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "empty_spec_equals_no_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: an empty format spec equals no spec for str: f'{x}', f'{x:}', f'{x!s:}' all give 'test' and f'{x!r:}' gives "'test'" for x='test'"""
# {x:} dispatches __format__('') just like {x}

x = "test"
assert f"{x}" == "test"
assert f"{x:}" == "test"          # empty spec is the same as no spec
assert f"{x!s:}" == "test"
assert f"{x!r:}" == "'test'"
# Built-in numbers ignore an empty spec and stringify normally.
assert f"{3:}" == "3"
assert f"{3!s:}" == "3"

print("empty_spec_equals_no_spec OK")
