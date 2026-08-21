# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "docxmlrpc"
# dimension = "behavior"
# case = "doc_xmlrpc_http_get_server__test_valid_get_response"
# subject = "cpython.test_docxmlrpc.DocXMLRPCHTTPGETServer.test_valid_get_response"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_docxmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_docxmlrpc.py::DocXMLRPCHTTPGETServer::test_valid_get_response
"""Auto-ported test: DocXMLRPCHTTPGETServer::test_valid_get_response."""


from xmlrpc.server import DocXMLRPCServer
import http.client
import threading


def make_server():
    serv = DocXMLRPCServer(("localhost", 0), logRequests=False)

    try:
        serv.set_server_title("DocXMLRPCServer Test Documentation")
        serv.set_server_name("DocXMLRPCServer Test Docs")
        serv.set_server_documentation(
            "This is an XML-RPC server's documentation, but the server "
            "can be used by POSTing to /RPC2. Try self.add, too."
        )

        class TestClass(object):
            def test_method(self, arg):
                """Test method's docs. This method truly does very little."""
                self.arg = arg

        serv.register_introspection_functions()
        serv.register_instance(TestClass())

        def add(x, y):
            """Add two instances together."""
            return x + y

        serv.register_function(add)
        return serv
    except BaseException:
        serv.server_close()
        raise


DocXMLRPCServer._send_traceback_header = True
server = make_server()
thread = threading.Thread(target=server.serve_forever)
thread.start()

port = server.server_address[1]
client = http.client.HTTPConnection("localhost", port, timeout=10)

try:
    client.request("GET", "/")
    response = client.getresponse()

    assert response.status == 200, response.status
    assert response.getheader("Content-type") == "text/html; charset=UTF-8"

    response.read()
finally:
    client.close()
    DocXMLRPCServer._send_traceback_header = False
    server.shutdown()
    thread.join()
    server.server_close()

print("DocXMLRPCHTTPGETServer::test_valid_get_response: ok")
