# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ossaudiodev"
# dimension = "errors"
# case = "oss_audio_dev_tests__test_on_closed"
# subject = "cpython.test_ossaudiodev.OSSAudioDevTests.test_on_closed"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ossaudiodev.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ossaudiodev.py::OSSAudioDevTests::test_on_closed
"""Auto-ported test: OSSAudioDevTests::test_on_closed (CPython 3.12 oracle)."""


import errno


try:
    import ossaudiodev
except ImportError:
    print("OSSAudioDevTests::test_on_closed: skipped, ossaudiodev unavailable")
    raise SystemExit(0)


def assert_raises_value_error(func, *args):
    try:
        func(*args)
    except ValueError:
        return
    raise AssertionError(f"{func.__name__}{args!r} did not raise ValueError")


try:
    dsp = ossaudiodev.open("w")
except (ossaudiodev.error, OSError) as exc:
    if exc.args and exc.args[0] in (errno.EACCES, errno.ENOENT, errno.ENODEV, errno.EBUSY):
        print(f"OSSAudioDevTests::test_on_closed: skipped, OSS dsp unavailable: {exc}")
        raise SystemExit(0)
    raise

dsp.close()
assert_raises_value_error(dsp.fileno)
assert_raises_value_error(dsp.read, 1)
assert_raises_value_error(dsp.write, b"x")
assert_raises_value_error(dsp.writeall, b"x")
assert_raises_value_error(dsp.bufsize)
assert_raises_value_error(dsp.obufcount)
assert_raises_value_error(dsp.obuffree)
assert_raises_value_error(dsp.getptr)

try:
    mixer = ossaudiodev.openmixer()
except (ossaudiodev.error, OSError) as exc:
    if exc.args and exc.args[0] in (errno.EACCES, errno.ENOENT, errno.ENODEV, errno.EBUSY):
        print(f"OSSAudioDevTests::test_on_closed: skipped, OSS mixer unavailable: {exc}")
        raise SystemExit(0)
    raise

mixer.close()
assert_raises_value_error(mixer.fileno)

print("OSSAudioDevTests::test_on_closed: ok")
