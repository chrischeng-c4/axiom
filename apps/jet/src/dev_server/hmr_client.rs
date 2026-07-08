// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
/// Generate the full HMR client runtime JavaScript.
///
/// This replaces the minimal `generate_hmr_client()` IIFE with a comprehensive
/// runtime that supports:
/// - `import.meta.hot` API (accept/dispose/prune/invalidate/data)
/// - Module registry keyed by URL
/// - WebSocket message handlers: update, css-update, full-reload, error, prune
/// - Dynamic `import()` with cache-busting timestamp query
/// - Error overlay: dark backdrop, monospace, code frame, click/Escape dismiss
/// - Exponential backoff reconnection (1s, 2s, 4s, ... max 30s)
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn generate_hmr_runtime() -> String {
    r#"<script type="module">
// ─── Jet HMR Runtime ─────────────────────────────────────────────────────────
(function() {
  if (typeof window === 'undefined') return;

  // ── Module Registry ──────────────────────────────────────────────────────
  // Map<moduleUrl, { accept, acceptDeps, dispose, prune, data, invalidate }>
  const moduleRegistry = new Map();

  // ── import.meta.hot factory ──────────────────────────────────────────────
  window.__jet_hmr_create_hot = function(moduleUrl) {
    let entry = moduleRegistry.get(moduleUrl);
    if (!entry) {
      entry = {
        acceptSelf: null,
        acceptDeps: new Map(),
        disposeCb: null,
        pruneCb: null,
        invalidateCalled: false,
        data: {},
      };
      moduleRegistry.set(moduleUrl, entry);
    }

    return {
      get data() { return entry.data; },

      accept(depsOrCb, cb) {
        if (typeof depsOrCb === 'function' || depsOrCb === undefined) {
          // Self-accepting: accept() or accept(cb)
          entry.acceptSelf = depsOrCb || true;
        } else if (Array.isArray(depsOrCb) && typeof cb === 'function') {
          // Dependency accept: accept(['./dep'], cb)
          for (const dep of depsOrCb) {
            const resolved = new URL(dep, moduleUrl).pathname;
            entry.acceptDeps.set(resolved, cb);
          }
        }
      },

      dispose(cb) {
        if (typeof cb === 'function') {
          entry.disposeCb = cb;
        }
      },

      prune(cb) {
        if (typeof cb === 'function') {
          entry.pruneCb = cb;
        }
      },

      invalidate() {
        entry.invalidateCalled = true;
      },
    };
  };

  // ── Error Overlay ────────────────────────────────────────────────────────
  let overlayContainer = null;

  function createOverlayContainer() {
    if (overlayContainer) return overlayContainer;
    overlayContainer = document.createElement('div');
    overlayContainer.id = '__jet_error_overlay';
    overlayContainer.style.cssText = [
      'position: fixed',
      'top: 0',
      'left: 0',
      'width: 100vw',
      'height: 100vh',
      'background: rgba(0, 0, 0, 0.85)',
      'z-index: 99999',
      'display: flex',
      'flex-direction: column',
      'align-items: center',
      'justify-content: center',
      'overflow-y: auto',
      'padding: 24px',
      'box-sizing: border-box',
      'font-family: "SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", monospace',
      'color: #fff',
    ].join(';');

    overlayContainer.addEventListener('click', (e) => {
      if (e.target === overlayContainer) dismissOverlay();
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') dismissOverlay();
    });

    document.body.appendChild(overlayContainer);
    return overlayContainer;
  }

  function showError(err) {
    const container = createOverlayContainer();
    container.style.display = 'flex';

    const card = document.createElement('div');
    card.style.cssText = [
      'background: #1a1a2e',
      'border: 1px solid #e94560',
      'border-radius: 8px',
      'padding: 20px 24px',
      'max-width: 800px',
      'width: 100%',
      'margin-bottom: 12px',
      'box-shadow: 0 4px 24px rgba(233, 69, 96, 0.3)',
    ].join(';');

    let html = '<div style="color:#e94560;font-size:14px;font-weight:600;margin-bottom:8px;">Error</div>';

    if (err.file) {
      let loc = err.file;
      if (err.line != null) loc += ':' + err.line;
      if (err.column != null) loc += ':' + err.column;
      html += '<div style="color:#aaa;font-size:12px;margin-bottom:12px;">' + escapeHtml(loc) + '</div>';
    }

    html += '<div style="color:#f0f0f0;font-size:13px;white-space:pre-wrap;margin-bottom:12px;">' + escapeHtml(err.message) + '</div>';

    if (err.frame) {
      html += '<pre style="background:#0d1117;border-radius:4px;padding:12px;font-size:12px;overflow-x:auto;color:#c9d1d9;margin:0;">' + escapeHtml(err.frame) + '</pre>';
    }

    card.innerHTML = html;
    // Newest on top
    container.insertBefore(card, container.firstChild);
  }

  function dismissOverlay() {
    if (overlayContainer) {
      overlayContainer.style.display = 'none';
      overlayContainer.innerHTML = '';
    }
  }

  function escapeHtml(str) {
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  // ── CSS Update ───────────────────────────────────────────────────────────
  function handleCssUpdate(msg) {
    // Find existing <link> or <style> with matching filename pattern
    const links = document.querySelectorAll('link[rel="stylesheet"]');
    for (const link of links) {
      const href = link.getAttribute('href') || '';
      // Match by base name (before hash)
      const baseName = msg.filename.replace(/\.[a-f0-9]+\.css$/, '');
      if (href.includes(baseName) || href.includes(msg.filename)) {
        link.href = '/' + msg.filename + '?t=' + msg.timestamp;
        return;
      }
    }

    // Check for <style data-jet-css> injected styles
    const jetStyles = document.querySelectorAll('style[data-jet-css]');
    if (jetStyles.length > 0) {
      jetStyles[jetStyles.length - 1].textContent = msg.css;
      return;
    }

    // Create new <style> element
    const style = document.createElement('style');
    style.setAttribute('data-jet-css', msg.filename);
    style.textContent = msg.css;
    document.head.appendChild(style);
  }

  // ── Module Hot Update ────────────────────────────────────────────────────
  async function handleUpdate(msg) {
    const { path, timestamp, acceptedBy } = msg;
    const moduleUrl = acceptedBy || path;

    const entry = moduleRegistry.get(moduleUrl);
    if (!entry && !acceptedBy) {
      // Module not registered in HMR — full reload as safety net
      console.log('[Jet] Module not in HMR registry, reloading:', path);
      window.location.reload();
      return;
    }

    // 1. Run dispose callback on the old module
    if (entry && entry.disposeCb) {
      try {
        entry.disposeCb(entry.data);
      } catch (e) {
        console.error('[Jet] dispose callback error:', e);
      }
    }

    // 2. Re-import the changed module with cache-busting
    try {
      const newModule = await import(path + '?t=' + timestamp);

      // 3. Run accept callback
      if (entry) {
        if (acceptedBy && entry.acceptDeps.has(path)) {
          // Dependency accept — parent handles update
          const cb = entry.acceptDeps.get(path);
          if (typeof cb === 'function') {
            cb([newModule]);
          }
        } else if (entry.acceptSelf) {
          // Self-accepting
          if (typeof entry.acceptSelf === 'function') {
            entry.acceptSelf(newModule);
          }
        }
      }

      // 4. Dismiss error overlay on successful update
      dismissOverlay();

      console.log('[Jet] Hot updated:', path);
    } catch (e) {
      console.error('[Jet] HMR update failed, falling back to reload:', e);
      window.location.reload();
    }
  }

  // ── Prune ────────────────────────────────────────────────────────────────
  function handlePrune(msg) {
    for (const path of msg.paths) {
      const entry = moduleRegistry.get(path);
      if (entry && entry.pruneCb) {
        try {
          entry.pruneCb();
        } catch (e) {
          console.error('[Jet] prune callback error:', e);
        }
      }
      moduleRegistry.delete(path);
    }
  }

  // ── Console Error Relay ──────────────────────────────────────────────────
  let relayWs = { current: null };
  let relayHooked = false;

  function setupConsoleRelay(ws) {
    relayWs.current = ws;
    if (relayHooked) return;
    relayHooked = true;

    function send(level, message, stack, url, line, column) {
      const s = relayWs.current;
      if (s && s.readyState === WebSocket.OPEN) {
        s.send(JSON.stringify({
          type: 'console-report',
          level: level,
          message: String(message),
          stack: stack || null,
          url: url || null,
          line: typeof line === 'number' ? line : null,
          column: typeof column === 'number' ? column : null,
          timestamp: Date.now()
        }));
      }
    }

    const origError = console.error;
    console.error = function(...args) {
      const text = args.map(String).join(' ');
      if (!text.startsWith('[Jet]')) {
        send('error', text, new Error().stack);
      }
      origError.apply(console, args);
    };

    const origWarn = console.warn;
    console.warn = function(...args) {
      const text = args.map(String).join(' ');
      if (!text.startsWith('[Jet]')) {
        send('warn', text, new Error().stack);
      }
      origWarn.apply(console, args);
    };

    window.addEventListener('error', (e) => {
      send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);
    });

    window.addEventListener('unhandledrejection', (e) => {
      const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);
      const stack = e.reason instanceof Error ? e.reason.stack : null;
      send('error', 'Unhandled rejection: ' + msg, stack);
    });
  }

  // ── WebSocket Connection ─────────────────────────────────────────────────
  let retryDelay = 1000;
  const MAX_RETRY_DELAY = 30000;

  function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const ws = new WebSocket(protocol + '//' + host + '/__jet_hmr');

    ws.onopen = () => {
      console.log('[Jet] HMR connected');
      retryDelay = 1000; // Reset backoff on successful connection
      setupConsoleRelay(ws);
    };

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      switch (msg.type) {
        case 'connected':
          console.log('[Jet] HMR ready');
          break;

        case 'update':
          console.log('[Jet] Module update:', msg.path);
          handleUpdate(msg);
          break;

        case 'css-update':
          console.log('[Jet] CSS update:', msg.filename);
          handleCssUpdate(msg);
          break;

        case 'full-reload':
          console.log('[Jet] Full reload:', msg.reason);
          window.location.reload();
          break;

        case 'error':
          console.error('[Jet] Error:', msg.message);
          showError(msg);
          break;

        case 'prune':
          console.log('[Jet] Pruning modules:', msg.paths);
          handlePrune(msg);
          break;

        default:
          console.log('[Jet] Unknown HMR message:', msg);
      }
    };

    ws.onerror = () => {
      // Error will be followed by close — reconnect handled there
    };

    ws.onclose = () => {
      console.log('[Jet] HMR disconnected. Reconnecting in ' + retryDelay + 'ms...');
      setTimeout(() => {
        retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY);
        connect();
      }, retryDelay);
    };
  }

  console.log('[Jet] Connecting to HMR server...');
  connect();
})();
</script>
"#
    .to_string()
}

/// Generate the `import.meta.hot` injection code for a served JS module.
///
/// Prepends a small snippet that creates the `import.meta.hot` object keyed
/// by the module's URL path.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn generate_hot_preamble(module_url: &str) -> String {
    format!(
        "if (window.__jet_hmr_create_hot) {{ import.meta.hot = window.__jet_hmr_create_hot(\"{}\"); }}\n",
        module_url
    )
}
// CODEGEN-END
