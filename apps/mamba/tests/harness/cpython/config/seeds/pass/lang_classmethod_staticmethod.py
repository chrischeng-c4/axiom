# Operational AssertionPass seed for @classmethod and @staticmethod
# descriptors on a class.
# Surface: @classmethod receives the class as `cls` and can mutate
# class-level state across calls; @staticmethod produces a callable
# that ignores both `self` and `cls`. Only the access-via-class form
# is asserted for @staticmethod — access-via-instance currently
# returns None on mamba (gap tracked separately).
class Counter:
    count = 0

    @classmethod
    def increment(cls):
        cls.count += 1
        return cls.count

    @staticmethod
    def hello():
        return "hello"

_ledger: list[int] = []
# @classmethod: cls receives the class object; state mutates monotonically
a = Counter.increment()
b = Counter.increment()
c = Counter.increment()
assert a == 1; _ledger.append(1)
assert b == 2; _ledger.append(1)
assert c == 3; _ledger.append(1)
# Class-level state mutated by the @classmethod across calls
assert Counter.count == 3; _ledger.append(1)
# @staticmethod: callable via the class, ignores self/cls
assert Counter.hello() == "hello"; _ledger.append(1)
# Re-calling @classmethod continues to mutate the same class state
d = Counter.increment()
assert d == 4; _ledger.append(1)
assert Counter.count == 4; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_classmethod_staticmethod {sum(_ledger)} asserts")
