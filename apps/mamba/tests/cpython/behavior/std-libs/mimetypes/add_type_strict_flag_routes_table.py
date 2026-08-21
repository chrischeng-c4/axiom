# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "add_type_strict_flag_routes_table"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: add_type(strict=) routes to the strict vs common table: default add_type lands in the strict table, strict=False in the loose one, and guess_all_extensions(strict=False) sees both"""
import mimetypes

db = mimetypes.MimeTypes()

# add_type default lands in the strict table; strict=False in the loose one.
db.add_type("test-type", ".strict-ext")
db.add_type("test-type", ".non-strict-ext", strict=False)
assert db.guess_all_extensions("test-type") == [".strict-ext"], "default strict only"
assert db.guess_all_extensions("test-type", strict=False) == [
    ".strict-ext",
    ".non-strict-ext",
], "loose sees both"
print("add_type_strict_flag_routes_table OK")
