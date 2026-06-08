# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "clear_frames_empties_frame_locals"
# subject = "traceback.clear_frames"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.clear_frames: clear_frames(tb) empties each traceback frame's f_locals: an innermost frame with one local has 0 locals after the call"""
import traceback


def _outer():
    _middle()


def _middle():
    _inner()


def _inner():
    _i = 1
    1 / 0


try:
    _outer()
except ZeroDivisionError as e:
    _tb = e.__traceback__
_innermost = _tb.tb_next.tb_next.tb_next.tb_frame
assert len(_innermost.f_locals) == 1, f"locals before clear = {len(_innermost.f_locals)!r}"
traceback.clear_frames(_tb)
assert len(_innermost.f_locals) == 0, f"locals after clear = {len(_innermost.f_locals)!r}"

print("clear_frames_empties_frame_locals OK")
