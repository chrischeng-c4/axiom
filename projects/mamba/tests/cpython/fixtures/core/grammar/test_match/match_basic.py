# RUN: parse
# CPython 3.12 test_match: basic structural pattern matching (PEP 634)

# Literal patterns
def http_status(status):
    match status:
        case 200:
            return "OK"
        case 404:
            return "Not Found"
        case 500:
            return "Internal Server Error"
        case _:
            return "Unknown"

# Variable capture
match (1, 2):
    case (x, y):
        pass

# OR patterns
match 42:
    case 1 | 2 | 3:
        pass
    case _:
        pass

# Guard clause
match 10:
    case x if x > 5:
        pass
    case x:
        pass

# String patterns
match "hello":
    case "hello":
        pass
    case "world":
        pass

# None / True / False patterns
match None:
    case None:
        pass
    case True:
        pass
    case False:
        pass

# Nested tuple patterns
match (1, (2, 3)):
    case (a, (b, c)):
        pass
