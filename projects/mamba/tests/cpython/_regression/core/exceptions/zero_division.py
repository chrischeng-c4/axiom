# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: division by zero must raise ZeroDivisionError catchable
# from Python. mb_div was already raising internally but the module-level
# entry point didn't surface pending exceptions, so uncaught cases
# silently exited 0.

try:
    x = 1 / 0
except ZeroDivisionError as e:
    print("caught:", e)

print("alive")