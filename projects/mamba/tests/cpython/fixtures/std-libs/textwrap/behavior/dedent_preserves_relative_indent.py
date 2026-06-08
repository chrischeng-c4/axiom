# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_preserves_relative_indent"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: uneven (code-block) indentation keeps relative indentation after the common prefix is removed"""
import textwrap

code = "        def foo():\n            while 1:\n                return foo\n        "
assert textwrap.dedent(code) == "def foo():\n    while 1:\n        return foo\n", (
    f"code = {textwrap.dedent(code)!r}"
)
nested = "  Foo\n    Bar\n \n   Baz\n"
assert textwrap.dedent(nested) == "Foo\n  Bar\n\n Baz\n", (
    f"nested = {textwrap.dedent(nested)!r}"
)
print("dedent_preserves_relative_indent OK")
