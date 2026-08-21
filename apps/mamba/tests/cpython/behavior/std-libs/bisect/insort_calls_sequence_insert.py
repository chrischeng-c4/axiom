# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "insort_calls_sequence_insert"
# subject = "bisect.insort_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.insort_left: insort uses the sequence's own insert(); a list subclass overriding insert observes the call"""
import bisect


# insort calls the sequence's own insert(): a list subclass that overrides
# insert observes the insertion (here it redirects into .store).
class TrackingList(list):
    def __init__(self):
        super().__init__()
        self.store = []

    def insert(self, index, item):
        self.store.insert(index, item)


lst = TrackingList()
bisect.insort_left(lst, 10)
bisect.insort_right(lst, 5)
assert lst.store == [5, 10], f"list-subclass insert = {lst.store!r}"

print("insort_calls_sequence_insert OK")
