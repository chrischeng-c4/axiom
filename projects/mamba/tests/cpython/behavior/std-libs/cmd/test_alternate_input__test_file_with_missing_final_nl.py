# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd"
# dimension = "behavior"
# case = "test_alternate_input__test_file_with_missing_final_nl"
# subject = "cpython.test_cmd.TestAlternateInput.test_file_with_missing_final_nl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestAlternateInput.test_file_with_missing_final_nl: cmdloop handles EOF without final newline."""

import cmd
import io


class SimpleCmd(cmd.Cmd):
    def do_print(self, args):
        print(args, file=self.stdout)

    def do_EOF(self, args):
        return True


input_stream = io.StringIO("print test\nprint test2")
output_stream = io.StringIO()
shell = SimpleCmd(stdin=input_stream, stdout=output_stream)
shell.use_rawinput = False
shell.cmdloop()

assert output_stream.getvalue() == "(Cmd) test\n(Cmd) test2\n(Cmd) ", output_stream.getvalue()

print("TestAlternateInput::test_file_with_missing_final_nl: ok")
