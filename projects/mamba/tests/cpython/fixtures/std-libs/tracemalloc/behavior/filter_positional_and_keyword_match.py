# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "filter_positional_and_keyword_match"
# subject = "tracemalloc.Filter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: full positional Filter(False, 'test.py', 123, True) equals the keyword-built Filter field-for-field"""
import tracemalloc

# Full positional construction.
f = tracemalloc.Filter(False, "test.py", 123, True)
assert f.inclusive is False, "inclusive False"
assert f.filename_pattern == "test.py", "pattern test.py"
assert f.lineno == 123, "lineno 123"
assert f.all_frames is True, "all_frames True"

# Keyword construction matches positional.
g = tracemalloc.Filter(
    inclusive=False, filename_pattern="test.py", lineno=123, all_frames=True
)
assert (g.inclusive, g.filename_pattern, g.lineno, g.all_frames) == (
    False,
    "test.py",
    123,
    True,
), "keyword construction"

print("filter_positional_and_keyword_match OK")
