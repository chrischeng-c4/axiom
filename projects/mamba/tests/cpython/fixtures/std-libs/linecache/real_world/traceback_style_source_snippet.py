# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "real_world"
# case = "traceback_style_source_snippet"
# subject = "linecache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache: a traceback-style tool writes a source file, then uses checkcache + getline to print a context snippet (the failing line plus the lines above and below), the way traceback/inspect render frames"""
import linecache
import tempfile
import os

SOURCE = (
    "def divide(a, b):\n"      # 1
    "    quotient = a / b\n"    # 2  <- the 'failing' line
    "    return quotient\n"     # 3
    "\n"                        # 4
    "result = divide(1, 0)\n"   # 5
)

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "frame.py")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(SOURCE)

    # A traceback renderer points at the offending line and shows context.
    fail_lineno = 2
    # checkcache makes sure a stale cached entry is refreshed before reading.
    linecache.checkcache(path)

    snippet = []
    for lineno in range(fail_lineno - 1, fail_lineno + 2):
        marker = "->" if lineno == fail_lineno else "  "
        text = linecache.getline(path, lineno).rstrip("\n")
        snippet.append(f"{marker} {lineno} {text}")

    assert snippet == [
        "   1 def divide(a, b):",
        "-> 2     quotient = a / b",
        "   3     return quotient",
    ], snippet

    # The cache now holds the file we rendered.
    assert path in linecache.cache, "rendered file is cached"

print("traceback_style_source_snippet OK")
