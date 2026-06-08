# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "read_mime_types_parses_table"
# subject = "mimetypes.read_mime_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.read_mime_types: read_mime_types parses a 'type ext ext' rules file into a {'.ext': 'type'} dict, mapping each extension to its type and skipping '# comment' lines"""
import mimetypes
import os
import tempfile

# read_mime_types parses 'type ext...' lines (and skips '# comments').
text = "application/x-foo  foo bar\n# comment line\napplication/x-baz  baz\n"
with tempfile.NamedTemporaryFile("w", suffix=".types", delete=False) as fh:
    fh.write(text)
    name = fh.name
try:
    table = mimetypes.read_mime_types(name)
finally:
    os.unlink(name)

assert table[".foo"] == "application/x-foo", f".foo = {table.get('.foo')!r}"
assert table[".bar"] == "application/x-foo", f".bar = {table.get('.bar')!r}"
assert table[".baz"] == "application/x-baz", f".baz = {table.get('.baz')!r}"
print("read_mime_types_parses_table OK")
