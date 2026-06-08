# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Top-level `def` is hoisted into HirModule.functions and never flows
# through `HirStmt::FuncDefPlaceholder`, so `mb_func_set_name` was never
# emitted for top-level user functions. `f.__name__` therefore returned
# `None` instead of the source-level identifier. Now primed at module
# init from a `user_func_names` map.

def foo(): pass

def bar(x, y): return x + y

print(foo.__name__)            # foo
print(bar.__name__)            # bar
print(repr(foo.__name__))      # 'foo'

# Functions defined later still register at module init.
def baz(): return 42
print(baz.__name__)            # baz
