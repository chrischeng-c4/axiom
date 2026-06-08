# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "real_world"
# case = "select_source_files_in_listing"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: a realistic flow: given a directory listing, use fnmatch.filter to select the Python source files (*.py) while skipping other extensions, mirroring how a build tool picks inputs"""
import fnmatch

# A build tool scans a directory listing and selects only the Python sources.
listing = [
    "setup.py",
    "README.md",
    "pkg/__init__.py",
    "pkg/core.py",
    "pkg/data.json",
    "pkg/core.pyc",
    "tests/test_core.py",
    "Makefile",
]

sources = fnmatch.filter(listing, "*.py")
assert sources == [
    "setup.py",
    "pkg/__init__.py",
    "pkg/core.py",
    "tests/test_core.py",
], f"selected sources = {sources!r}"

# Order is preserved and non-.py entries (including the .pyc) are skipped.
assert all(name.endswith(".py") for name in sources), "only .py selected"
assert "pkg/core.pyc" not in sources, ".pyc is not a .py source"

print("select_source_files_in_listing OK")
