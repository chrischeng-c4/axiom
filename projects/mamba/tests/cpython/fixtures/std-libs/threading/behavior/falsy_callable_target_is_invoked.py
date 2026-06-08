# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "falsy_callable_target_is_invoked"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: the target is checked for 'is not None', not truthiness, so a callable whose __bool__ is False is still invoked"""
import threading

class _FalsyCallable:
    def __init__(self):
        self.ran = False
    def __bool__(self):
        return False
    def __call__(self):
        self.ran = True

_falsy = _FalsyCallable()
tf = threading.Thread(target=_falsy)
tf.start()
tf.join()
assert _falsy.ran, "falsy callable target was invoked"

print("falsy_callable_target_is_invoked OK")
