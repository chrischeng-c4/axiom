# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_empty_string_returns_empty_list"
# subject = "textwrap.wrap"
# kind = "semantic"
# xfail = "textwrap.wrap is a silent stub under mamba — returns input rather than [] (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.wrap: wrap on the empty string returns an empty list"""
import textwrap

assert textwrap.wrap("") == [], f"wrap('') = {textwrap.wrap('')!r}"
print("wrap_empty_string_returns_empty_list OK")
