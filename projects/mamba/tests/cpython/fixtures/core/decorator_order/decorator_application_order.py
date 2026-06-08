# Decorator application order — #2778.
#
# Covers Python's decorator application order for functions and
# classes. Decorators in Python apply *bottom-up* (the decorator
# closest to `def`/`class` runs first) but call-wrapping happens
# *top-down* (outermost decorator runs first when the decorated
# object is invoked).
#
# Clauses:
#   1. Bottom-up application — two function decorators record the
#      order in which they wrap the underlying function. `@outer`
#      above `@inner` above `def f` applies inner first, outer
#      second.
#   2. Top-down call wrapping — when the decorated function is
#      called, the outermost decorator's wrapper runs first.
#   3. Three-decorator stack confirms order generalizes.
#   4. Identity sentinel proves the decorated object is the wrapper
#      returned by the outermost decorator (not the original
#      function).
#   5. Class decorator application order — class decorators follow
#      the same bottom-up rule.
#   6. Decorator factory (decorator-returning-decorator) preserves
#      the application order rule.
#
# Every print line tagged `[decorator-order]` so failure output
# names decorator semantics.

# Trace buffer — every decorator writes to this to make the order
# observable. `apply` events fire at def-time; `enter` / `exit`
# fire at call-time.
TRACE = []


def make(label):
    """Return a decorator that records its apply + call ordering."""

    def deco(f):
        TRACE.append(f"apply:{label}")

        def wrapper(*a, **kw):
            TRACE.append(f"enter:{label}")
            result = f(*a, **kw)
            TRACE.append(f"exit:{label}")
            return result

        # Stamp the wrapper so identity assertions can prove this is
        # the OUTERMOST decorator's wrapper, not the original fn.
        wrapper._mamba_decorator_label = label  # type: ignore[attr-defined]
        return wrapper

    return deco


# Clause 1 + 2 + 4: two-decorator stack.
TRACE.clear()

@make("outer")
@make("inner")
def double(x):
    TRACE.append("body")
    return x * 2

# `apply` order proves bottom-up: inner first, outer second.
print("[decorator-order] clause-1 apply-order:", [e for e in TRACE if e.startswith("apply:")])

# Call the function; `enter`/`exit` order proves top-down call
# wrapping: outer enters first, exits last.
TRACE.clear()
got = double(21)
print("[decorator-order] clause-2 call-result:", got)
print("[decorator-order] clause-2 call-order:", TRACE)

# Identity: the function-name `double` resolves to the OUTERMOST
# wrapper.
print(
    "[decorator-order] clause-4 outermost-label:",
    getattr(double, "_mamba_decorator_label", None),
)


# Clause 3: three decorators — confirm pattern generalizes.
TRACE.clear()

@make("a")
@make("b")
@make("c")
def triple(x):
    TRACE.append("body")
    return x + 100

print("[decorator-order] clause-3 apply-order:", TRACE[:])
TRACE.clear()
print("[decorator-order] clause-3 call-result:", triple(1))
print("[decorator-order] clause-3 call-order:", TRACE[:])


# Clause 5: class decorator.
CLASS_TRACE = []


def cls_make(label):
    def deco(cls):
        CLASS_TRACE.append(f"apply:{label}")
        cls._mamba_decorator_label = label  # type: ignore[attr-defined]
        return cls

    return deco


@cls_make("CO")
@cls_make("CI")
class Decorated:
    pass


# Class decorators apply bottom-up too; CI before CO.
print("[decorator-order] clause-5 apply-order:", CLASS_TRACE)
# Outermost decorator wins the stamp because it overwrites the
# attribute last.
print(
    "[decorator-order] clause-5 outermost-label:",
    getattr(Decorated, "_mamba_decorator_label", None),
)


# Clause 6: decorator factory — a decorator that takes args and
# returns a real decorator. Application order rule still holds.
def tagged(name):
    def deco(f):
        TRACE.append(f"factory-apply:{name}")

        def wrapper(*a, **kw):
            TRACE.append(f"factory-enter:{name}")
            return f(*a, **kw)

        return wrapper

    return deco


TRACE.clear()

@tagged("OUT")
@tagged("IN")
def fact(x):
    return x

print("[decorator-order] clause-6 apply-order:", TRACE[:])
TRACE.clear()
fact(42)
print("[decorator-order] clause-6 enter-order:", TRACE[:])
