# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_binary_attrs_flip_on_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a binary spool reports mode 'w+b'/name None before rollover and mode 'rb+'/a real name after; text-only attrs (newlines/encoding/errors) raise AttributeError"""
import tempfile

f = tempfile.SpooledTemporaryFile(max_size=10)
f.write(b"x" * 10)
assert not f._rolled, "still in memory at exactly max_size"
assert f.mode == "w+b", f"pre-rollover mode = {f.mode!r}"
assert f.name is None, f"pre-rollover name = {f.name!r}"
for attr in ("newlines", "encoding", "errors"):
    _raised = False
    try:
        getattr(f, attr)
    except AttributeError:
        _raised = True
    assert _raised, f"binary spool should not expose {attr}"
f.write(b"x")  # exceed max_size -> roll to disk
assert f._rolled, "rolled after exceeding max_size"
assert f.mode == "rb+", f"post-rollover mode = {f.mode!r}"
assert f.name is not None, "rolled spool has a real name"
f.close()
print("spooled_binary_attrs_flip_on_rollover OK")
