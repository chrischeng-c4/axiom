# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "attrgetter_dotted_and_multi"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter reads a single attribute, walks a dotted path through nested objects, and with multiple names returns a tuple in argument order"""
import operator

class Node:
    pass


root = Node()
root.name = "arthur"
root.child = Node()
root.child.name = "thomas"
root.child.child = Node()
root.child.child.name = "johnson"

assert operator.attrgetter("name")(root) == "arthur", "single attr"
assert operator.attrgetter("child.name")(root) == "thomas", "one-level dotted"
assert operator.attrgetter("child.child.name")(root) == "johnson", "two-level dotted"

get_multi = operator.attrgetter("name", "child.name", "child.child.name")
assert get_multi(root) == ("arthur", "thomas", "johnson"), "multi dotted tuple"

print("attrgetter_dotted_and_multi OK")
