# RUN: parse
# CPython 3.12 test_match: sequence patterns with star unpacking

# Simple sequence
match [1, 2, 3]:
    case [1, 2, 3]:
        pass

# Star pattern (captures rest)
match [1, 2, 3, 4, 5]:
    case [1, *rest]:
        pass

match [1, 2, 3, 4, 5]:
    case [first, *middle, last]:
        pass

# Wildcard star
match [1, 2, 3]:
    case [1, *_]:
        pass

# Empty sequence
match []:
    case []:
        pass
    case [x]:
        pass
    case [x, *rest]:
        pass

# Nested sequences
match [[1, 2], [3, 4]]:
    case [[a, b], [c, d]]:
        pass

# Mixed with OR
match [1, 2]:
    case [1, 2] | [3, 4]:
        pass

# Tuple pattern
match (1, "hello"):
    case (int(n), str(s)):
        pass
