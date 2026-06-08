# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# try-except-else: else runs when no exception, skipped when exception

# Case 1: no exception — else runs
try:
    x = 10
except ValueError:
    print("caught")
else:
    print("no exception")

# Case 2: exception — else skipped
try:
    raise ValueError("oops")
except ValueError:
    print("caught ValueError")
else:
    print("this should not print")

# Case 3: else with computation
try:
    result = 100 // 5
except ZeroDivisionError:
    print("division error")
else:
    print(f"result is {result}")