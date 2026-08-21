# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_collapses_whitespace_and_truncates"
# subject = "textwrap.shorten"
# kind = "semantic"
# xfail = "textwrap.shorten is a silent stub under mamba — no whitespace-collapse/truncate (repo memory project-mamba-stdlib-stub-audit-2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten collapses runs of whitespace and truncates with the placeholder so the result fits within width"""
import textwrap

sh = textwrap.shorten("   hello   world   ", width=10, placeholder="...")
assert sh == "hello...", f"shorten = {sh!r}"
assert len(sh) <= 10, f"shorten len = {len(sh)!r}"
# A longer sentence truncates with the placeholder and stays within width.
sh2 = textwrap.shorten("hello world this is long", width=15, placeholder="...")
assert len(sh2) <= 15, f"shorten len = {len(sh2)!r}"
assert sh2.endswith("..."), f"shorten ends with ... = {sh2!r}"
print("shorten_collapses_whitespace_and_truncates OK")
