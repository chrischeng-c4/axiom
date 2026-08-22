# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "safety"
# lib = "re"
# dimension = "concurrency"
# case = "match_group_in_child_does_not_corrupt_heap"
# subject = "re.Match.group"
# kind = "semantic"
# mem_carveout = ""
# source = "#3103"
# status = "filled"
# ///
"""ABSOLUTE clause: calling match.group() on a worker thread must not corrupt memory.

mamba dies here with a varying fatal signal -- SIGSEGV, SIGABRT, SIGBUS or
SIGTRAP (#3103). Observed 10 crashes in 10 runs of exactly this shape.

Deliberately NOT marked xfail. The runner classifies a negative return code as
CRASH regardless of the xfail flag and a CRASH always fails the suite, which is
the correct fail-closed treatment for memory unsafety. Marking it xfail would
imply this is an accepted gap; it is not.

DO NOT "TIDY" THIS FIXTURE. Three innocuous-looking edits each make the crash
vanish completely, turning this file into a green light over a live p0. Every
one of them was measured, not guessed:

1. **No `try`/`except` around the `group()` call.** Wrapping it passes 5/5 --
   the faulty release appears to sit on a lowering path the exception-handling
   path avoids.
2. **No `for` loop.** This is the counter-intuitive one. Putting the two
   `group()` calls in a `for _ in range(40)` loop passes 5/5, and so does
   spawning the thread inside a main-thread loop. Looping *destroys* the
   signal; the straight-line worker body below is what reproduces. An earlier
   revision of this fixture looped 40 times "for reliability" and reported
   PASS 3/3 while the bare probe was crashing on the same binary.
3. **Main must actually consume the captured strings.** Retaining them in a
   list and asserting only `len()` passes 5/5. The `captured != [...]`
   comparison below is load-bearing, not cosmetic.

`span()` and `bool(match)` on the same match are safe, and `re.sub`,
`re.findall`, `re.compile`, `re.search` itself, `str.upper`, concat and
`split()` are all safe in a child -- the fault is specific to the heap string
that the group accessor returns.
"""
import re
import threading

captured: list = []


def worker() -> None:
    match = re.search(r"(\d+)", "xy 77")
    # No try/except, and no loop. See the docstring -- both mask the bug.
    captured.append(match.group(1))
    captured.append(match.group(0))


t = threading.Thread(target=worker)
t.start()
t.join()

if captured != ["77", "77"]:
    print(f"concurrency: FAIL: child produced {captured!r}, expected ['77', '77']")
else:
    print("concurrency: PASS")
