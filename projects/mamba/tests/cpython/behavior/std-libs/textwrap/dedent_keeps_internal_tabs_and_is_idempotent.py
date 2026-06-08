# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_keeps_internal_tabs_and_is_idempotent"
# subject = "textwrap.dedent"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: internal (non-leading) tabs are preserved verbatim and dedent is idempotent on already-dedented text"""
import textwrap

tabbed = "  hello\tthere\n  how are\tyou?"
once = textwrap.dedent(tabbed)
assert once == "hello\tthere\nhow are\tyou?", f"tabbed = {once!r}"
assert textwrap.dedent(once) == once, "dedent idempotent on dedented text"
print("dedent_keeps_internal_tabs_and_is_idempotent OK")
