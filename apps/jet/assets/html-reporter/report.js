// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-assets-html-reporter.md#logic
// CODEGEN-BEGIN
/* Jet HTML Reporter — report.js */
(function() {
  'use strict';

  // Read data island injected by Rust side
  var dataEl = document.getElementById('report-data');
  var reportData = null;
  if (dataEl) {
    try { reportData = JSON.parse(dataEl.textContent); } catch(e) {}
  }

  var currentFilter = 'all';

  function badgeClass(status) {
    switch (status) {
      case 'passed': return 'badge-passed';
      case 'failed': return 'badge-failed';
      case 'skipped': return 'badge-skipped';
      case 'flaky': return 'badge-flaky';
      default: return 'badge-timedout';
    }
  }

  function escHtml(s) {
    if (!s) return '';
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function formatDiff(diff) {
    if (!diff) return '';
    return diff.split('\n').map(function(line) {
      if (line.startsWith('+')) return '<span class="diff-line-add">' + escHtml(line) + '</span>';
      if (line.startsWith('-')) return '<span class="diff-line-remove">' + escHtml(line) + '</span>';
      return '<span class="diff-line-ctx">' + escHtml(line) + '</span>';
    }).join('\n');
  }

  // GH #3073 — the file:// report has no backend that can intercept
  // ?trace=<path>, so navigating the URL bar was a silent no-op. Surface
  // the actionable command (`jet trace view <path>`) in an in-page toast
  // with copy-to-clipboard instead.
  function handleTraceLink(e) {
    e.preventDefault();
    var tracePath = this.dataset.tracePath;
    if (!tracePath) return;
    var cmd = 'jet trace view ' + tracePath;
    showTraceToast(cmd);
  }

  function showTraceToast(cmd) {
    var existing = document.getElementById('trace-toast');
    if (existing) existing.remove();

    var toast = document.createElement('div');
    toast.id = 'trace-toast';
    toast.className = 'trace-toast';
    toast.setAttribute('role', 'status');

    var label = document.createElement('div');
    label.className = 'trace-toast-label';
    label.textContent = 'Run this command to view the trace:';
    toast.appendChild(label);

    var code = document.createElement('code');
    code.className = 'trace-toast-cmd';
    code.textContent = cmd;
    toast.appendChild(code);

    var copy = document.createElement('button');
    copy.className = 'trace-toast-copy';
    copy.type = 'button';
    copy.textContent = 'Copy';
    copy.addEventListener('click', function() { copyToClipboard(cmd, copy); });
    toast.appendChild(copy);

    var close = document.createElement('button');
    close.className = 'trace-toast-close';
    close.type = 'button';
    close.setAttribute('aria-label', 'Close');
    close.textContent = '×';
    close.addEventListener('click', function() { toast.remove(); });
    toast.appendChild(close);

    document.body.appendChild(toast);
  }

  function copyToClipboard(text, btn) {
    function done() {
      if (!btn) return;
      var orig = btn.textContent;
      btn.textContent = 'Copied!';
      setTimeout(function() { btn.textContent = orig; }, 1500);
    }
    function fallback() {
      var ta = document.createElement('textarea');
      ta.value = text;
      ta.setAttribute('readonly', '');
      ta.style.position = 'absolute';
      ta.style.left = '-9999px';
      document.body.appendChild(ta);
      ta.select();
      try { document.execCommand('copy'); done(); } catch (_) {}
      ta.remove();
    }
    if (navigator.clipboard && window.isSecureContext) {
      navigator.clipboard.writeText(text).then(done, fallback);
    } else {
      fallback();
    }
  }

  function toggleDrawer(rowId) {
    var drawer = document.getElementById('drawer-' + rowId);
    if (drawer) {
      if (drawer.classList.contains('open')) {
        drawer.classList.remove('open');
      } else {
        drawer.classList.add('open');
      }
    }
  }

  function applyFilter(filter) {
    currentFilter = filter;
    var pills = document.querySelectorAll('.pill');
    pills.forEach(function(pill) {
      pill.classList.toggle('active', pill.dataset.filter === filter);
    });

    var rows = document.querySelectorAll('tr[data-status]');
    rows.forEach(function(row) {
      var status = row.dataset.status;
      var show = filter === 'all' || status === filter;
      row.style.display = show ? '' : 'none';

      // Also hide the corresponding drawer row
      var drawerId = row.dataset.drawerId;
      if (drawerId) {
        var drawerRow = document.getElementById('drawer-' + drawerId);
        if (drawerRow) {
          if (!show) {
            drawerRow.classList.remove('open');
            drawerRow.style.display = 'none';
          } else {
            drawerRow.style.display = '';
          }
        }
      }
    });
  }

  function init() {
    // Wire filter pills
    document.querySelectorAll('.pill').forEach(function(pill) {
      pill.addEventListener('click', function() {
        applyFilter(this.dataset.filter);
      });
    });

    // Wire toggle buttons
    document.querySelectorAll('.toggle-btn').forEach(function(btn) {
      btn.addEventListener('click', function() {
        toggleDrawer(this.dataset.rowId);
      });
    });

    // Wire trace links
    document.querySelectorAll('.trace-link').forEach(function(link) {
      link.addEventListener('click', handleTraceLink);
    });

    // GH #3073 — legacy ?trace= URL param handler removed; the in-page
    // toast (see handleTraceLink) replaces the silent navigation.
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
// CODEGEN-END
