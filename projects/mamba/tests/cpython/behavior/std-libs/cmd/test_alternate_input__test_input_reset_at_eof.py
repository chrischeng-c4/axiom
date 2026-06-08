# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "behavior"
# case = "test_alternate_input__test_input_reset_at_eof"
# subject = "cpython.test_cmd.TestAlternateInput.test_input_reset_at_EOF"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestAlternateInput.test_input_reset_at_EOF: cmdloop reuses reset stdin/stdout."""

import cmd
import io


class SimpleCmd(cmd.Cmd):
    def do_print(self, args):
        print(args, file=self.stdout)

    def do_EOF(self, args):
        return True


class SimpleCmd2(SimpleCmd):
    def do_EOF(self, args):
        print("*** Unknown syntax: EOF", file=self.stdout)
        return True


input_stream = io.StringIO("print test\nprint test2")
output_stream = io.StringIO()
shell = SimpleCmd2(stdin=input_stream, stdout=output_stream)
shell.use_rawinput = False
shell.cmdloop()
assert output_stream.getvalue() == (
    "(Cmd) test\n"
    "(Cmd) test2\n"
    "(Cmd) *** Unknown syntax: EOF\n"
), output_stream.getvalue()

input_stream = io.StringIO("print \n\n")
output_stream = io.StringIO()
shell.stdin = input_stream
shell.stdout = output_stream
shell.cmdloop()
assert output_stream.getvalue() == (
    "(Cmd) \n"
    "(Cmd) \n"
    "(Cmd) *** Unknown syntax: EOF\n"
), output_stream.getvalue()

print("TestAlternateInput::test_input_reset_at_EOF: ok")
