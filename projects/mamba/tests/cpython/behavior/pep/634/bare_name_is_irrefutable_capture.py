# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "bare_name_is_irrefutable_capture"
# subject = "match.capture_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.capture_pattern: a bare name is an irrefutable capture: it always matches and binds across value kinds"""

# A bare name is an irrefutable capture: it always matches and binds.
def capture_all(x):
    match x:
        case got:
            return got


assert capture_all(42) == 42
assert capture_all(None) is None
assert capture_all((1, 2)) == (1, 2)
print("bare_name_is_irrefutable_capture OK")
