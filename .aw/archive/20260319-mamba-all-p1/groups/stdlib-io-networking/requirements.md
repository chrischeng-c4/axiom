---
change: mamba-all-p1
group: stdlib-io-networking
date: 2026-03-19
---

# Requirements

Implement 6 native stdlib modules for I/O, networking, and concurrency:
- #658 selectors: `DefaultSelector`, `SelectSelector`, `register()`, `unregister()`, `select()` — backed by mio or tokio
- #661 ssl: `SSLContext`, `wrap_socket()`, `create_default_context()` — backed by rustls or native-tls
- #662 urllib: `urllib.parse` (urlparse, urlencode, quote, unquote), `urllib.request` (urlopen, Request), `urllib.error` (URLError, HTTPError)
- #663 email: `message_from_string()`, `EmailMessage`, `MIMEText`, `MIMEMultipart`, header parsing
- #664 multiprocessing: `Process`, `Pool`, `Queue`, `Pipe`, `Value`, `Array` — backed by std::process
- #665 concurrent.futures: `ThreadPoolExecutor`, `ProcessPoolExecutor`, `Future`, `as_completed()`, `wait()` — backed by tokio/rayon
All modules must integrate with Mamba's async runtime (tokio) where appropriate and expose CPython-compatible APIs.
