# Language conformance: try/finally cleanup and context managers (P0-R2).
# Tests try/finally, try/except/finally, and with-statement protocol.

# Basic try/finally
print("test 1")
try:
    print("try block")
finally:
    print("finally block")

# try/except/finally with exception
print("test 2")
try:
    raise ValueError("oops")
except ValueError:
    print("caught")
finally:
    print("done")

# try/except/finally without exception
print("test 3")
try:
    x = 10
    print(x)
except ValueError:
    print("not reached")
finally:
    print("cleanup")
