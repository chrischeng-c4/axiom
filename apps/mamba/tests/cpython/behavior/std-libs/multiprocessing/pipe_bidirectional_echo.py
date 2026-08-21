# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "behavior"
# case = "pipe_bidirectional_echo"
# subject = "multiprocessing.Pipe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Pipe: a Pipe gives two Connection ends; the parent sends 'hello', the child recvs it, sends back data*2, and the parent recvs 'hellohello' (spawn-guarded under __main__)"""
import multiprocessing


def _pipe_echo(conn):
    data = conn.recv()
    conn.send(data * 2)
    conn.close()


if __name__ == "__main__":
    parent_conn, child_conn = multiprocessing.Pipe()
    p = multiprocessing.Process(target=_pipe_echo, args=(child_conn,))
    p.start()
    parent_conn.send("hello")
    response = parent_conn.recv()
    p.join(timeout=10)
    assert response == "hellohello", f"pipe echo = {response!r}"
    parent_conn.close()

    print("pipe_bidirectional_echo OK")
