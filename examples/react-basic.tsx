/**
 * Basic React Example
 *
 * This example demonstrates how to use RuSheet as a React component.
 *
 * Installation:
 * ```bash
 * npm install rusheet react react-dom
 * ```
 */

import { useRef, useState } from 'react';
import { RuSheet, RuSheetRef, useRuSheet, CellChangeEvent } from 'rusheet/react';

/**
 * Example 1: Basic Usage with Ref
 */
export function BasicExample() {
  const sheetRef = useRef<RuSheetRef>(null);
  const [lastChange, setLastChange] = useState<string>('');

  const handleChange = (event: CellChangeEvent) => {
    setLastChange(`Cell ${event.col},${event.row} = "${event.value}"`);
  };

  const handleAddRow = () => {
    sheetRef.current?.insertRows(0, 1);
  };

  const handleSetValue = () => {
    sheetRef.current?.setCellValue(0, 0, 'Hello, RuSheet!');
  };

  return (
    <div>
      <h2>Basic RuSheet Example</h2>

      <div style={{ marginBottom: 16, display: 'flex', gap: 8 }}>
        <button onClick={handleAddRow}>Insert Row at Top</button>
        <button onClick={handleSetValue}>Set A1 Value</button>
      </div>

      {lastChange && <p>Last change: {lastChange}</p>}

      <RuSheet
        ref={sheetRef}
        initialData={[
          ['Name', 'Age', 'City'],
          ['Alice', 30, 'New York'],
          ['Bob', 25, 'Los Angeles'],
          ['Charlie', 35, 'Chicago'],
        ]}
        onChange={handleChange}
        width="100%"
        height={400}
      />
    </div>
  );
}

/**
 * Example 2: Using the useRuSheet Hook
 */
export function HookExample() {
  const { ref, api } = useRuSheet();
  const [data, setData] = useState<string>('');

  const handleExport = () => {
    const json = api.serialize();
    setData(json);
  };

  const handleBold = () => {
    api.setCellFormat(0, 0, { bold: true });
  };

  return (
    <div>
      <h2>useRuSheet Hook Example</h2>

      <div style={{ marginBottom: 16, display: 'flex', gap: 8 }}>
        <button onClick={handleBold}>Bold A1</button>
        <button onClick={handleExport}>Export JSON</button>
        <button onClick={() => api.undo()}>Undo</button>
        <button onClick={() => api.redo()}>Redo</button>
      </div>

      <RuSheet
        ref={ref}
        initialData={[
          ['Product', 'Price', 'Quantity', 'Total'],
          ['Widget', 10, 5, '=B2*C2'],
          ['Gadget', 25, 3, '=B3*C3'],
          ['Total', '', '', '=SUM(D2:D3)'],
        ]}
        width="100%"
        height={300}
      />

      {data && (
        <details style={{ marginTop: 16 }}>
          <summary>Exported JSON</summary>
          <pre style={{ fontSize: 12, overflow: 'auto', maxHeight: 200 }}>
            {data}
          </pre>
        </details>
      )}
    </div>
  );
}

/**
 * Example 3: Controlled Data
 */
export function ControlledExample() {
  const { ref, api } = useRuSheet();
  const [rows, setRows] = useState(3);

  const handleAddRow = () => {
    api.insertRows(rows, 1);
    setRows((r) => r + 1);
  };

  const handleRemoveRow = () => {
    if (rows > 1) {
      api.deleteRows(rows - 1, 1);
      setRows((r) => r - 1);
    }
  };

  const handleReset = () => {
    api.setData([
      ['A', 'B', 'C'],
      [1, 2, 3],
      [4, 5, 6],
    ]);
    setRows(3);
  };

  return (
    <div>
      <h2>Controlled Data Example</h2>

      <div style={{ marginBottom: 16, display: 'flex', gap: 8 }}>
        <button onClick={handleAddRow}>Add Row</button>
        <button onClick={handleRemoveRow}>Remove Row</button>
        <button onClick={handleReset}>Reset Data</button>
        <span>Rows: {rows}</span>
      </div>

      <RuSheet
        ref={ref}
        initialData={[
          ['A', 'B', 'C'],
          [1, 2, 3],
          [4, 5, 6],
        ]}
        showFormulaBar={false}
        width={400}
        height={250}
      />
    </div>
  );
}

/**
 * Example 4: Read-Only Mode
 */
export function ReadOnlyExample() {
  return (
    <div>
      <h2>Read-Only Spreadsheet</h2>
      <p>This spreadsheet cannot be edited by the user.</p>

      <RuSheet
        initialData={[
          ['Report', 'Q1', 'Q2', 'Q3', 'Q4'],
          ['Revenue', 1000, 1200, 1100, 1400],
          ['Expenses', 800, 850, 900, 950],
          ['Profit', '=B2-B3', '=C2-C3', '=D2-D3', '=E2-E3'],
        ]}
        readOnly={true}
        showFormulaBar={false}
        width="100%"
        height={200}
      />
    </div>
  );
}

/**
 * Example 5: With Collaboration
 */
export function CollaborationExample() {
  const [connected, setConnected] = useState(false);

  return (
    <div>
      <h2>Collaborative Editing</h2>
      <p>Status: {connected ? 'Connected' : 'Disconnected'}</p>

      <RuSheet
        initialData={[['Collaborative', 'Spreadsheet']]}
        collaboration={{
          serverUrl: 'ws://localhost:8080',
          roomId: 'demo-room',
          user: {
            id: 'user-' + Math.random().toString(36).slice(2, 9),
            name: 'Demo User',
            color: '#4CAF50',
          },
          onConnect: () => setConnected(true),
          onDisconnect: () => setConnected(false),
        }}
        width="100%"
        height={400}
      />
    </div>
  );
}

/**
 * Main App combining all examples
 */
export default function App() {
  return (
    <div style={{ padding: 24, maxWidth: 1200, margin: '0 auto' }}>
      <h1>RuSheet React Examples</h1>

      <section style={{ marginBottom: 48 }}>
        <BasicExample />
      </section>

      <section style={{ marginBottom: 48 }}>
        <HookExample />
      </section>

      <section style={{ marginBottom: 48 }}>
        <ControlledExample />
      </section>

      <section style={{ marginBottom: 48 }}>
        <ReadOnlyExample />
      </section>

      <section style={{ marginBottom: 48 }}>
        <CollaborationExample />
      </section>
    </div>
  );
}
