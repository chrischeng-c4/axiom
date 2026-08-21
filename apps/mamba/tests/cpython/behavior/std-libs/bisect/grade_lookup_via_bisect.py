# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "grade_lookup_via_bisect"
# subject = "bisect.bisect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect: the classic grade-band lookup: bisect maps a score to its grade bucket"""
import bisect

grades = [60, 70, 80, 90]
marks = ['F', 'D', 'C', 'B', 'A']

def grade(score):
    return marks[bisect.bisect(grades, score)]

assert grade(55) == 'F', f"grade 55 = {grade(55)!r}"
assert grade(75) == 'C', f"grade 75 = {grade(75)!r}"
assert grade(95) == 'A', f"grade 95 = {grade(95)!r}"

print("grade_lookup_via_bisect OK")
