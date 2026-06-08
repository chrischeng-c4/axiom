# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_suffix_map_tgz"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: suffix_map collapses .tgz/.svgz: backup.tgz and archive.svgz resolve through the suffix table to (type, encoding)"""
import mimetypes

# suffix_map expands a shorthand suffix into its long form before lookup.
assert mimetypes.suffix_map[".tgz"] == ".tar.gz", mimetypes.suffix_map.get(".tgz")
assert mimetypes.suffix_map[".svgz"] == ".svg.gz", mimetypes.suffix_map.get(".svgz")

assert mimetypes.guess_type("backup.tgz") == ("application/x-tar", "gzip"), \
    mimetypes.guess_type("backup.tgz")
assert mimetypes.guess_type("archive.svgz") == ("image/svg+xml", "gzip"), \
    mimetypes.guess_type("archive.svgz")
print("guess_type_suffix_map_tgz OK")
