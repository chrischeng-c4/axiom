# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "behavior"
# case = "st_field_indices"
# subject = "stat.ST_MODE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.ST_MODE: stat-result field index constants carry the documented positions: ST_MODE==0, ST_SIZE==6, ST_MTIME==8"""
import stat

# Field-index constants index into an os.stat_result tuple.
assert stat.ST_MODE == 0, "ST_MODE"
assert stat.ST_SIZE == 6, "ST_SIZE"
assert stat.ST_MTIME == 8, "ST_MTIME"

print("st_field_indices OK")
