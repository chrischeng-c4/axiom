# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "only_taken_branch_binds"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: captures from an unmatched case are not bound; only the taken branch binds names"""

# Captures from an unmatched case are NOT bound; only the taken branch binds.
def which_branch(v):
    taken = None
    match v:
        case 0:
            taken = "literal"
        case [head, *_]:
            taken = ("seq", head)
        case other:
            taken = ("capture", other)
    return taken


assert which_branch(0) == "literal"
assert which_branch([7, 8]) == ("seq", 7)
assert which_branch("z") == ("capture", "z")
print("only_taken_branch_binds OK")
