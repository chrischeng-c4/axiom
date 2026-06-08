# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# try-except-else-finally: all clauses interact correctly

# Case 1: no exception — else + finally both run
try:
    x = 42
except Exception:
    print("caught")
else:
    print("else ran")
finally:
    print("finally ran")

# Case 2: exception — except + finally, no else
try:
    raise RuntimeError("fail")
except RuntimeError:
    print("caught RuntimeError")
else:
    print("else should not run")
finally:
    print("finally always runs")

# Case 3: nested try-except-else
try:
    try:
        val = int("123")
    except ValueError:
        print("inner caught")
    else:
        print(f"inner else: {val}")
except Exception:
    print("outer caught")
else:
    print("outer else")