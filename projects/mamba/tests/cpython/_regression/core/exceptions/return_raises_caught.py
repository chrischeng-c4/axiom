# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: `return <expr>` where <expr> raises must be catchable
# by the enclosing try/except. Previously the Return terminator didn't
# check for a pending exception, so the function quietly returned None
# and the exception surfaced past the except clause.

def parse_or_default():
    try:
        return int("not a number")
    except ValueError:
        return -1

print(parse_or_default())

# Nested try with return-raising
def nested():
    try:
        try:
            return int("bad")
        except ValueError:
            raise RuntimeError("rewrapped")
    except RuntimeError as e:
        return str(e)

print(nested())

# Catch via tuple
def multi():
    try:
        return int("x")
    except (ValueError, TypeError) as e:
        return type(e).__name__

print(multi())

# Return raising inside a for loop's try
def find():
    for s in ["1", "abc", "2"]:
        try:
            return int(s)
        except ValueError:
            continue
    return -1

print(find())