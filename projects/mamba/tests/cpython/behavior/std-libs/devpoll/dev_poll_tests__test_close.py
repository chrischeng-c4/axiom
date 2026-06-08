# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "devpoll"
# dimension = "behavior"
# case = "dev_poll_tests__test_close"
# subject = "cpython.test_devpoll.DevPollTests.test_close"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_devpoll.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_devpoll.py::DevPollTests::test_close
"""Auto-ported test: DevPollTests::test_close (CPython 3.12 oracle)."""


import select


if not hasattr(select, "devpoll"):
    print("DevPollTests::test_close: skipped, select.devpoll unavailable")
    raise SystemExit(0)

with open(__file__, "rb") as open_file:
    fd = open_file.fileno()
    devpoll = select.devpoll()

    assert isinstance(devpoll.fileno(), int), devpoll.fileno()
    assert not devpoll.closed

    devpoll.close()
    assert devpoll.closed

    try:
        devpoll.fileno()
    except ValueError:
        pass
    else:
        raise AssertionError("devpoll.fileno() on closed devpoll did not raise ValueError")

    devpoll.close()

    for operation in (
        lambda: devpoll.modify(fd, select.POLLIN),
        lambda: devpoll.poll(),
        lambda: devpoll.register(fd, select.POLLIN),
        lambda: devpoll.unregister(fd),
    ):
        try:
            operation()
        except ValueError:
            pass
        else:
            raise AssertionError("closed devpoll operation did not raise ValueError")

print("DevPollTests::test_close: ok")
