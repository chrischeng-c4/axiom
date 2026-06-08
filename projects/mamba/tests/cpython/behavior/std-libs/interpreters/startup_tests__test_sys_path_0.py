# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "startup_tests__test_sys_path_0"
# subject = "cpython.test_interpreters.StartupTests.test_sys_path_0"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_interpreters.py::StartupTests::test_sys_path_0
"""Auto-ported test: StartupTests::test_sys_path_0 (CPython 3.12 oracle)."""


import contextlib
import json
import os
import os.path
import sys
import threading
from textwrap import dedent
import unittest
import time
from test import support
from test.support import import_helper
from test.support import threading_helper
from test.support import os_helper
from test.support import interpreters


_interpreters = import_helper.import_module('_xxsubinterpreters')

_channels = import_helper.import_module('_xxinterpchannels')

def _captured_script(script):
    r, w = os.pipe()
    indented = script.replace('\n', '\n                ')
    wrapped = dedent(f"\n        import contextlib\n        with open({w}, 'w', encoding='utf-8') as spipe:\n            with contextlib.redirect_stdout(spipe):\n                {indented}\n        ")
    return (wrapped, open(r, encoding='utf-8'))

def clean_up_interpreters():
    for interp in interpreters.list_all():
        if interp.id == 0:
            continue
        try:
            interp.close()
        except RuntimeError:
            pass

def _run_output(interp, request, channels=None):
    script, rpipe = _captured_script(request)
    with rpipe:
        interp.run(script, channels=channels)
        return rpipe.read()

@contextlib.contextmanager
def _running(interp):
    r, w = os.pipe()

    def run():
        interp.run(dedent(f'\n            # wait for "signal"\n            with open({r}) as rpipe:\n                rpipe.read()\n            '))
    t = threading.Thread(target=run)
    t.start()
    yield
    with open(w, 'w') as spipe:
        spipe.write('done')
    t.join()


# --- test body ---
def _debug(msg):
    if not self__debugged:
        print()
        self__debugged = True
    if self__subtest is not None:
        if True:
            if not self__debugged_in_subtest:
                self__debugged_in_subtest = True
                print('### start subtest debug ###')
            print(msg)
    else:
        print(msg)

def create_temp_dir():
    import tempfile
    tmp = tempfile.mkdtemp(prefix='test_interpreters_')
    tmp = os.path.realpath(tmp)
    pass
    return tmp

def debug(msg, *, header=None):
    if header:
        _debug(f'--- {header} ---')
        if msg:
            if msg.endswith(os.linesep):
                _debug(msg[:-len(os.linesep)])
            else:
                _debug(msg)
                _debug('<no newline>')
        _debug('------')
    else:
        _debug(msg)

def os_pipe():
    r, w = os.pipe()

    def cleanup():
        try:
            os.close(w)
        except Exception:
            pass
        try:
            os.close(r)
        except Exception:
            pass
    pass
    return (r, w)

def run_python(argv, *, cwd=None):
    import shlex
    import subprocess
    if isinstance(argv, str):
        argv = shlex.split(argv)
    argv = [sys.executable, *argv]
    try:
        proc = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    except Exception as exc:
        debug(f'# cmd: {shlex.join(argv)}')
        if isinstance(exc, FileNotFoundError) and (not exc.filename):
            if os.path.exists(argv[0]):
                exists = 'exists'
            else:
                exists = 'does not exist'
            debug(f'{argv[0]} {exists}')
        raise
    assert proc.stderr == '' or proc.returncode != 0, proc.stderr
    if proc.returncode != 0 and support.verbose:
        debug(f'# python3 {shlex.join(argv[1:])} failed:')
        debug(proc.stdout, header='stdout')
        debug(proc.stderr, header='stderr')

    assert proc.returncode == 0

    assert proc.stderr == ''
    return proc.stdout

def subTest(*args):
    with super().subTest(*args) as ctx:
        self__subtest_count += 1
        try:
            yield ctx
        finally:
            if self__debugged_in_subtest:
                if self__subtest_count == 1:
                    print('### end subtest debug ###', end='')
                else:
                    print('### end subtest debug ###')
            self__debugged_in_subtest = False

def write_script(*path, text):
    filename = os.path.join(*path)
    dirname = os.path.dirname(filename)
    if dirname:
        os.makedirs(dirname, exist_ok=True)
    with open(filename, 'w', encoding='utf-8') as outfile:
        outfile.write(dedent(text))
    return filename
script = '\n            import sys\n            from test.support import interpreters\n\n            orig = sys.path[0]\n\n            interp = interpreters.create()\n            interp.run(f"""if True:\n                import json\n                import sys\n                print(json.dumps({{\n                    \'main\': {orig!r},\n                    \'sub\': sys.path[0],\n                }}, indent=4), flush=True)\n                """)\n            '
cwd = create_temp_dir()
write_script(cwd, 'pkg', '__init__.py', text='')
write_script(cwd, 'pkg', '__main__.py', text=script)
write_script(cwd, 'pkg', 'script.py', text=script)
write_script(cwd, 'script.py', text=script)
cases = [('script.py', cwd), ('-m script', cwd), ('-m pkg', cwd), ('-m pkg.script', cwd), ('-c "import script"', '')]
for argv, expected in cases:
    out = run_python(argv, cwd=cwd)
    data = json.loads(out)
    sp0_main, sp0_sub = (data['main'], data['sub'])

    assert sp0_sub == sp0_main

    assert sp0_sub == expected
print("StartupTests::test_sys_path_0: ok")
