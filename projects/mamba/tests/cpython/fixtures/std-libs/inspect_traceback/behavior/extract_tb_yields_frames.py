# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "extract_tb_yields_frames"
# subject = "traceback.extract_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.extract_tb: inside an except block, traceback.extract_tb(e.__traceback__) walks the live traceback and yields at least one FrameSummary"""
import traceback

try:
    raise ValueError("boom")
except ValueError as exc:
    frames = traceback.extract_tb(exc.__traceback__)
    assert len(frames) >= 1, len(frames)

print("extract_tb_yields_frames OK")
