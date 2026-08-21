# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "init_resets_registry"
# subject = "mimetypes.init"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.init: init() rebuilds the global registry from defaults, discarding add_type edits: a registered foo/bar disappears after init()"""
import mimetypes

mimetypes.add_type("foo/bar", ".foobar")
assert mimetypes.guess_extension("foo/bar") == ".foobar", "add_type took effect"
mimetypes.init()
assert mimetypes.guess_extension("foo/bar") is None, "init() reset add_type"
print("init_resets_registry OK")
