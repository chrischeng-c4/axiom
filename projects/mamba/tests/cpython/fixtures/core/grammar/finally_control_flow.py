# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: return/break/continue inside finally (CPython 3.12 oracle).

Distilled from CPython GrammarTests: a finally clause that returns, breaks,
or continues overrides whatever the try block was doing -- including a
pending return value or a propagating exception.

CPython 3.12 emits a compile-time SyntaxWarning for these constructs (to
stderr) but still executes them with the documented semantics; stdout stays
clean and the process exits 0.
"""


# finally-return overrides a normal try body.
def g1():
    try:
        pass
    finally:
        return 1


# finally-return overrides try-return.
def g2():
    try:
        return 2
    finally:
        return 3


# finally-return swallows an exception raised in try.
def g3():
    try:
        1 / 0
    finally:
        return 4


assert g1() == 1
assert g2() == 3
assert g3() == 4
print("return_in_finally: ok")

# break in finally swallows the exception and exits the loop after one pass.
count = 0
while count < 5:
    count += 1
    try:
        1 / 0
    finally:
        break
assert count == 1
print("break_in_finally: ok")

# break in finally wins even over a continue in the try body.
count = 0
while count < 5:
    count += 1
    try:
        continue
    finally:
        break
assert count == 1
print("break_over_continue: ok")

# continue in finally resumes the loop, swallowing the try exception.
count = 0
while count < 3:
    count += 1
    try:
        1 / 0
    finally:
        continue
    count += 100  # unreachable: continue jumped back to the loop test
assert count == 3
print("continue_in_finally: ok")

# continue in finally also overrides a break in the try body.
count = 0
while count < 3:
    count += 1
    try:
        break
    finally:
        continue
assert count == 3
print("finally_control_flow OK")
