# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression test: hypothesis-g — retain_if_ptr for instance in mb_super
# Bug: super_proxy drop over-released instance (rc 1→0) before mb_raise_instance
# Fix: retain_if_ptr(instance) before fields.insert("__super_self__")

class AppError(Exception):
    def __init__(self, msg, code):
        super().__init__(msg)
        self.code = code

# Raise and catch — verifies instance survives super_proxy drop
try:
    raise AppError("not found", 404)
except AppError as e:
    print("caught:", e)
    print("code:", e.code)

# Attribute must survive after super().__init__ call
err = AppError("server error", 500)
print("msg:", str(err))
print("code:", err.code)