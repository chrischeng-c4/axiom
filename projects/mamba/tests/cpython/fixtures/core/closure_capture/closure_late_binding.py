# Closure late-binding — #2779.
#
# Covers Python's closure capture semantics. Python closures capture
# *variables*, not values — when the inner function runs it looks up
# the current value of the captured name, not the value at the time
# the closure was created. This is the famous "loop variable capture"
# gotcha; the canonical workaround is to bind the value into a
# default argument, which freezes it at function-definition time.
#
# Clauses:
#   1. Late binding — closures created in a loop all see the FINAL
#      value of the loop variable, not the value at creation time.
#   2. Default-arg capture — using `i=i` in the signature freezes
#      the value at each closure's def-time.
#   3. Rebinding the captured name AFTER closure creation propagates
#      to the closure — confirms it is the binding that is captured.
#   4. Free variable in enclosing scope reads the live value at call
#      time, not the value when the inner function was defined.
#   5. `nonlocal` write-back — the inner function's reassignment
#      mutates the enclosing scope's binding.
#   6. Multiple closures over the same cell share state.
#
# Every print line tagged `[closure]` so failure output names closure
# semantics.

# Clause 1: late binding — every lambda captures `i` by reference,
# so after the loop ends all of them see i = 4.
closures = []
for i in range(5):
    closures.append(lambda: i)

print("[closure] clause-1 late-binding:", [c() for c in closures])


# Clause 2: default-arg capture — `i=i` freezes the value at the
# moment each lambda is defined.
frozen = []
for i in range(5):
    frozen.append(lambda i=i: i)

print("[closure] clause-2 default-arg:", [c() for c in frozen])


# Clause 3: closures capture the binding, not the value. Mutate the
# captured name AFTER the closure is created and the change is
# visible.
def outer3():
    x = 1
    inner = lambda: x  # noqa: E731
    x = 99  # type: ignore[assignment]  # rebind AFTER inner was defined
    return inner()

print("[closure] clause-3 rebind-after:", outer3())


# Clause 4: enclosing scope mutation visible via closure. The two
# inner functions both close over `box`; mutating it through one is
# observable through the other.
def stateful_counter():
    box = [0]

    def read():
        return box[0]

    def bump():
        box[0] += 1

    return read, bump


read, bump = stateful_counter()
print("[closure] clause-4 read-initial:", read())
bump()
bump()
bump()
print("[closure] clause-4 read-after-bump:", read())


# Clause 5: `nonlocal` write-back.
def make_accumulator():
    total = 0

    def add(n):
        nonlocal total
        total += n
        return total

    def get():
        return total

    return add, get


add, get = make_accumulator()
add(1)
add(2)
add(3)
print("[closure] clause-5 nonlocal-add-result:", get())


# Clause 6: multiple closures over the same cell share state. Two
# inner functions both reading/writing `counter` see each other's
# mutations.
def shared_cell():
    counter = 0

    def inc():
        nonlocal counter
        counter += 1
        return counter

    def dec():
        nonlocal counter
        counter -= 1
        return counter

    return inc, dec


inc, dec = shared_cell()
print("[closure] clause-6 inc:", inc())
print("[closure] clause-6 inc:", inc())
print("[closure] clause-6 dec:", dec())
print("[closure] clause-6 inc:", inc())
