# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "context_manager_tracks_closed"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: an open TarFile reports .closed False, .close() flips it to True, and re-entering an already-closed TarFile as a context manager raises OSError"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("a.txt")
    _ti.size = 1
    _tf.addfile(_ti, io.BytesIO(b"x"))
_buf.seek(0)

_tf = tarfile.open(fileobj=_buf, mode="r")
assert not _tf.closed, "open archive not closed"
_tf.close()
assert _tf.closed, "closed archive is closed"

_raised = False
try:
    with _tf:
        pass
except OSError:
    _raised = True
assert _raised, "reusing closed TarFile raises OSError"

print("context_manager_tracks_closed OK")
