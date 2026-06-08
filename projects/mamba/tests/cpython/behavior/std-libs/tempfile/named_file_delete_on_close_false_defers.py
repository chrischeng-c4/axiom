# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_delete_on_close_false_defers"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: 3.12 delete_on_close=False with delete=True: the file persists after an explicit close() and is removed only at with-block exit"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d2:
    name2 = ""
    with tempfile.NamedTemporaryFile(dir=d2, delete=True,
                                     delete_on_close=False) as dc:
        dc.write(b"blat")
        name2 = dc.name
        dc.close()
        assert os.path.exists(name2), "still present after close()"
    assert not os.path.exists(name2), "removed at context-manager exit"
print("named_file_delete_on_close_false_defers OK")
