# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_propagates_input_exceptions"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() does not swallow exceptions raised by its input iterators; an IndexError from a faulty generator surfaces to the caller"""
import heapq


# This generator over-indexes its backing list and raises IndexError, which
# must surface to the caller instead of being silently dropped.
def faulty():
    backing = list(range(5))
    for i in range(10):  # i reaches 5..9 -> IndexError
        yield backing[i]


raised = False
try:
    list(heapq.merge(faulty(), faulty()))
except IndexError:
    raised = True
assert raised, "merge must propagate IndexError from inputs"
print("merge_propagates_input_exceptions OK")
