# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "polar_rect_round_trip"
# subject = "cmath.polar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.polar: polar and rect are inverse operations across a table of representative complex points"""
import cmath

for _x, _y in [(1, 0), (0, 1), (-1, 0), (1, 1), (3, 4)]:
    _z = complex(_x, _y)
    _r, _phi = cmath.polar(_z)
    _z2 = cmath.rect(_r, _phi)
    assert abs(_z - _z2) < 1e-12, f"polar/rect round-trip {_z}"
print("polar_rect_round_trip OK")
