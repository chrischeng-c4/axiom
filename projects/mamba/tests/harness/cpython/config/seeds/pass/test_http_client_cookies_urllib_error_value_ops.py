# Atomic 310 pass conformance — http.client module (hasattr HTTP
# Connection/HTTPSConnection/HTTPResponse/HTTPException/OK/NOT_FOUND/
# INTERNAL_SERVER_ERROR/responses/HTTPS_PORT/HTTP_PORT + OK == 200 +
# NOT_FOUND == 404 + INTERNAL_SERVER_ERROR == 500 + HTTP_PORT == 80 +
# HTTPS_PORT == 443) + http.cookies module (hasattr SimpleCookie/Base
# Cookie/Morsel/CookieError + type(SimpleCookie()).__name__ ==
# 'SimpleCookie') + urllib.error module (hasattr URLError/HTTPError/
# ContentTooShortError).
# All asserts match between CPython 3.12 and mamba.
from http import client as http_client
from http import cookies as http_cookies
from urllib import error as urllib_error


_ledger: list[int] = []

# 1) http.client — hasattr core surface
assert hasattr(http_client, "HTTPConnection") == True; _ledger.append(1)
assert hasattr(http_client, "HTTPSConnection") == True; _ledger.append(1)
assert hasattr(http_client, "HTTPResponse") == True; _ledger.append(1)
assert hasattr(http_client, "HTTPException") == True; _ledger.append(1)
assert hasattr(http_client, "OK") == True; _ledger.append(1)
assert hasattr(http_client, "NOT_FOUND") == True; _ledger.append(1)
assert hasattr(http_client, "INTERNAL_SERVER_ERROR") == True; _ledger.append(1)
assert hasattr(http_client, "responses") == True; _ledger.append(1)
assert hasattr(http_client, "HTTPS_PORT") == True; _ledger.append(1)
assert hasattr(http_client, "HTTP_PORT") == True; _ledger.append(1)

# 2) http.client — value contracts (HTTPStatus IntEnum compares to int)
assert http_client.OK == 200; _ledger.append(1)
assert http_client.NOT_FOUND == 404; _ledger.append(1)
assert http_client.INTERNAL_SERVER_ERROR == 500; _ledger.append(1)
assert http_client.HTTP_PORT == 80; _ledger.append(1)
assert http_client.HTTPS_PORT == 443; _ledger.append(1)

# 3) http.cookies — hasattr core surface + type contract
assert hasattr(http_cookies, "SimpleCookie") == True; _ledger.append(1)
assert hasattr(http_cookies, "BaseCookie") == True; _ledger.append(1)
assert hasattr(http_cookies, "Morsel") == True; _ledger.append(1)
assert hasattr(http_cookies, "CookieError") == True; _ledger.append(1)
assert type(http_cookies.SimpleCookie()).__name__ == "SimpleCookie"; _ledger.append(1)

# 4) urllib.error — hasattr core surface
assert hasattr(urllib_error, "URLError") == True; _ledger.append(1)
assert hasattr(urllib_error, "HTTPError") == True; _ledger.append(1)
assert hasattr(urllib_error, "ContentTooShortError") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_http_client_cookies_urllib_error_value_ops {sum(_ledger)} asserts")
