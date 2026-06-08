/**
 * Grid Example — Standalone Spreadsheet Demo
 *
 * A minimal example demonstrating the RuSheet grid engine:
 * - WASM-powered spreadsheet engine
 * - Canvas-based grid rendering
 * - Formula support (SUM, etc.)
 * - Cell editing, formatting, undo/redo
 * - Sheet tabs, insert row/col, CSV export
 */

import '../../src/styles/main.css';
import { rusheet } from '../../src/core/RusheetAPI';
import GridRenderer from '../../src/canvas/GridRenderer';
import type { IGridRenderer } from '../../src/types/renderer';
import InputController from '../../src/ui/InputController';
import CellEditor from '../../src/ui/CellEditor';

/** Convert column index to Excel-style letter (0→A, 1→B, …, 26→AA) */
function colToLetter(col: number): string {
  let result = '';
  let n = col;
  while (n >= 0) {
    result = String.fromCharCode(65 + (n % 26)) + result;
    n = Math.floor(n / 26) - 1;
    if (n < 0) break;
  }
  return result;
}

/** Export current sheet as CSV and trigger download */
function exportCSV(): void {
  const rows: string[][] = [];
  // Scan up to 100x26 for demo purposes
  for (let r = 0; r < 100; r++) {
    const row: string[] = [];
    let hasValue = false;
    for (let c = 0; c < 26; c++) {
      const cell = rusheet.getCellData(r, c);
      const val = cell?.displayValue ?? '';
      if (val) hasValue = true;
      row.push(val);
    }
    if (!hasValue && r > 0) break;
    rows.push(row);
  }

  const csv = rows
    .map(r => r.map(v => (v.includes(',') || v.includes('"') ? `"${v.replace(/"/g, '""')}"` : v)).join(','))
    .join('\n');

  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'spreadsheet.csv';
  a.click();
  URL.revokeObjectURL(url);
}

async function main(): Promise<void> {
  // 1. Initialize WASM engine
  await rusheet.init();

  // 2. Grab DOM elements
  const canvas = document.getElementById('spreadsheet-canvas') as HTMLCanvasElement;
  const formulaInput = document.getElementById('formula-input') as HTMLInputElement;
  const cellAddress = document.getElementById('cell-address') as HTMLSpanElement;
  const container = document.getElementById('spreadsheet-container') as HTMLElement;
  const addSheetBtn = document.getElementById('add-sheet-btn') as HTMLButtonElement;
  const statusText = document.getElementById('status-text') as HTMLSpanElement;

  if (!canvas || !formulaInput || !cellAddress || !container) {
    throw new Error('Required DOM elements not found');
  }

  // 3. Create renderer & editor
  const renderer: IGridRenderer = new GridRenderer(canvas);
  const cellEditor = new CellEditor(container, renderer, formulaInput);

  const editModeCallback = (row: number, col: number) => {
    cellEditor.activate(row, col);
  };
  new InputController(canvas, renderer, editModeCallback);

  // 4. Resize handler
  const resizeCanvas = () => {
    const rect = container.getBoundingClientRect();
    renderer.resize(rect.width, rect.height);
    cellEditor.updatePosition();
  };
  resizeCanvas();
  window.addEventListener('resize', resizeCanvas);

  // 5. Cell address display
  const updateUI = () => {
    const active = renderer.getActiveCell();
    cellAddress.textContent = `${colToLetter(active.col)}${active.row + 1}`;
    renderer.render();
  };

  // Re-render on filter changes
  rusheet.onFilterChange(() => renderer.render());

  // Update address + auto-save status on cell edits
  cellEditor.setCellChangeCallback((row: number, col: number) => {
    cellAddress.textContent = `${colToLetter(col)}${row + 1}`;
    updateUI();
  });

  // 6. Toolbar: Undo / Redo
  document.getElementById('undo-btn')?.addEventListener('click', () => {
    rusheet.undo('user');
    updateUI();
  });
  document.getElementById('redo-btn')?.addEventListener('click', () => {
    rusheet.redo('user');
    updateUI();
  });

  // 7. Toolbar: Bold / Italic
  document.getElementById('bold-btn')?.addEventListener('click', () => {
    const { row, col } = renderer.getActiveCell();
    const cell = rusheet.getCellData(row, col);
    const bold = !(cell?.format?.bold);
    rusheet.setRangeFormat(row, col, row, col, { bold }, 'user');
    updateUI();
  });
  document.getElementById('italic-btn')?.addEventListener('click', () => {
    const { row, col } = renderer.getActiveCell();
    const cell = rusheet.getCellData(row, col);
    const italic = !(cell?.format?.italic);
    rusheet.setRangeFormat(row, col, row, col, { italic }, 'user');
    updateUI();
  });

  // 8. Toolbar: Insert Row / Col
  document.getElementById('insert-row-btn')?.addEventListener('click', () => {
    const { row } = renderer.getActiveCell();
    rusheet.insertRows(row, 1, 'user');
    updateUI();
    if (statusText) statusText.textContent = `Inserted row at ${row + 1}`;
  });
  document.getElementById('insert-col-btn')?.addEventListener('click', () => {
    const { col } = renderer.getActiveCell();
    rusheet.insertCols(col, 1, 'user');
    updateUI();
    if (statusText) statusText.textContent = `Inserted column at ${colToLetter(col)}`;
  });

  // 9. Toolbar: Export CSV
  document.getElementById('export-btn')?.addEventListener('click', exportCSV);

  // 10. Sheet tabs
  let sheetCounter = 2;

  if (addSheetBtn) {
    addSheetBtn.addEventListener('click', () => {
      const name = `Sheet${sheetCounter}`;
      const idx = rusheet.addSheet(name, 'user');

      const tab = document.createElement('div');
      tab.className = 'sheet-tab';
      tab.setAttribute('data-index', String(idx));
      tab.textContent = name;

      document.querySelectorAll('.sheet-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      addSheetBtn.parentElement?.insertBefore(tab, addSheetBtn);

      rusheet.setActiveSheet(idx, 'user');
      sheetCounter++;
      updateUI();
    });
  }

  document.getElementById('sheet-tabs')?.addEventListener('click', (e) => {
    const target = e.target as HTMLElement;
    if (target.classList.contains('sheet-tab')) {
      const idx = parseInt(target.getAttribute('data-index') || '0', 10);
      document.querySelectorAll('.sheet-tab').forEach(t => t.classList.remove('active'));
      target.classList.add('active');
      rusheet.setActiveSheet(idx, 'user');
      updateUI();
    }
  });

  // 11. Populate sample data
  rusheet.setCellValue(0, 0, 'Product', 'api');
  rusheet.setCellValue(0, 1, 'Qty', 'api');
  rusheet.setCellValue(0, 2, 'Price', 'api');
  rusheet.setCellValue(0, 3, 'Total', 'api');

  const products = [
    ['Apples', '10', '1.50'],
    ['Oranges', '15', '2.00'],
    ['Bananas', '20', '0.75'],
    ['Mangoes', '8', '3.25'],
    ['Grapes', '12', '4.00'],
  ];
  products.forEach(([name, qty, price], i) => {
    const r = i + 1;
    rusheet.setCellValue(r, 0, name, 'api');
    rusheet.setCellValue(r, 1, qty, 'api');
    rusheet.setCellValue(r, 2, price, 'api');
    rusheet.setCellValue(r, 3, `=B${r + 1}*C${r + 1}`, 'api');
  });

  // Summary row
  const summaryRow = products.length + 1;
  rusheet.setCellValue(summaryRow, 0, 'Total:', 'api');
  rusheet.setCellValue(summaryRow, 3, `=SUM(D2:D${summaryRow})`, 'api');

  // Format header row (bold + grey background)
  rusheet.setRangeFormat(0, 0, 0, 3, { bold: true, backgroundColor: '#e8e8e8' }, 'api');
  // Format summary row (bold)
  rusheet.setRangeFormat(summaryRow, 0, summaryRow, 3, { bold: true }, 'api');

  // 12. Initial render
  updateUI();
  if (statusText) statusText.textContent = 'Ready';

  console.log('[Grid Example] Spreadsheet initialized');
}

main().catch((err) => {
  console.error('[Grid Example] Init failed:', err);
  const app = document.getElementById('app');
  if (app) {
    app.innerHTML = `
      <div style="display:flex;align-items:center;justify-content:center;height:100vh;flex-direction:column;font-family:sans-serif;">
        <h1 style="color:#d32f2f;">Failed to initialize Grid</h1>
        <p style="color:#666;margin-top:16px;">${err instanceof Error ? err.message : String(err)}</p>
        <p style="color:#999;margin-top:8px;font-size:12px;">Check the console for details. Make sure WASM is built: <code>pnpm build:wasm</code></p>
      </div>`;
  }
});
