# lang_match_literal_or.py - axis-1 PEP 634 match: literal + capture + or seed (#3343).
#
# Surface (from #3343):
#   1. Literal pattern matches int / str / None
#   2. Capture pattern binds name
#   3. Or pattern `1 | 2 | 3` matches alternatives
#   4. Wildcard `_` matches anything
#   5. Match expression returns from each arm

_ledger: list[int] = []


def classify(v):
    match v:
        case 0:
            return "zero-int"
        case "hi":
            return "greet-str"
        case None:
            return "none-literal"
        case 1 | 2 | 3:
            return "small-or"
        case bound:
            return ("captured", bound)


def wildcard_check(v):
    match v:
        case 42:
            return "forty-two"
        case _:
            return "wildcard"


# 1. Literal pattern: int.
assert classify(0) == "zero-int", "literal int 0 matches case 0:"
_ledger.append(1)

# 1. Literal pattern: str.
assert classify("hi") == "greet-str", "literal string 'hi' matches case 'hi':"
_ledger.append(1)

# 1. Literal pattern: None.
assert classify(None) == "none-literal", "literal None matches case None:"
_ledger.append(1)

# 3. Or pattern: each alternative matches.
assert classify(1) == "small-or", "or-pattern matches first alternative (1)"
_ledger.append(1)

assert classify(2) == "small-or", "or-pattern matches middle alternative (2)"
_ledger.append(1)

assert classify(3) == "small-or", "or-pattern matches last alternative (3)"
_ledger.append(1)

# 2. Capture pattern binds name on fallthrough.
res = classify(99)
assert res == ("captured", 99), "capture pattern binds the value into the named target"
_ledger.append(1)

# 4. Wildcard `_` matches anything outside enumerated arms.
assert wildcard_check(42) == "forty-two", "literal arm takes precedence over wildcard"
_ledger.append(1)

assert wildcard_check("anything") == "wildcard", "wildcard `_` matches non-literal value"
_ledger.append(1)

assert wildcard_check([1, 2, 3]) == "wildcard", "wildcard `_` matches arbitrary container"
_ledger.append(1)

# 5. Each arm exits via return (no fallthrough).
assert classify(0) != classify(1), "different arms return different values (no fallthrough)"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_match_literal_or {sum(_ledger)} asserts")
