# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "literal_first_match_wins"
# subject = "match.literal_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.literal_pattern: literal patterns compare by equality and the first matching case wins; str != int literal"""

# Literal patterns compare by equality; the first matching case wins.
def http_error(status):
    match status:
        case 400:
            return "bad"
        case 401 | 403 | 404:
            return "denied"
        case 418:
            return "teapot"
    return None  # no wildcard -> falls through to None


assert http_error(400) == "bad"
assert http_error(403) == "denied"
assert http_error(418) == "teapot"
assert http_error(123) is None
assert http_error("400") is None  # str does not equal int literal
print("literal_first_match_wins OK")
