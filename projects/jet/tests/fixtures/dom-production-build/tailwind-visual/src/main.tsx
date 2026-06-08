import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import "./main.css";

declare global {
  interface Window {
    __jetVisualFixture?: {
      libraryId: string;
      tableTitle: string;
      minComponentCases: number;
      primaryButtonText: string;
    };
    __jetVisualEvents?: string[];
  }
}

window.__jetVisualEvents = [];
window.addEventListener("error", (event) => {
  window.__jetVisualEvents?.push(String(event.error || event.message));
});
window.__jetVisualFixture = {
  libraryId: "tailwind",
  tableTitle: "Tailwind CSS visual table fixture",
  minComponentCases: 8,
  primaryButtonText: "Tailwind Primary",
};

const rows = [
  { id: 0, label: "cell 0", status: "ready" },
  ...Array.from({ length: 17 }, (_, index) => {
    const id = index + 1;
    return {
      id,
      label: `cell ${id}`,
      status: id % 2 === 0 ? "ready" : "queued",
    };
  }),
];

function App() {
  const [active, setActive] = useState(0);

  return (
    <main className="min-h-screen bg-slate-100 text-slate-900 p-6">
      <section className="component-matrix grid gap-4">
        <div className="tailwind-card ui-case">
          <p className="text-sm font-bold text-blue-700">Tailwind CSS visual table fixture</p>
          <h1 className="text-2xl font-bold">Tailwind component matrix</h1>
          <p className="text-slate-600">Native Jet Tailwind utilities and CSS layer output</p>
        </div>
        <div className="tailwind-card ui-case flex items-center gap-4">
          <button
            type="button"
            className="ui-case rounded bg-blue-500 px-4 py-2 text-white shadow"
            onClick={() => setActive((value) => value + 1)}
          >
            Tailwind Primary
          </button>
          <span className="ui-case text-sm text-slate-700">Tailwind count {active}</span>
        </div>
        <div className="tailwind-card ui-case grid grid-cols-3 gap-4">
          <span className="ui-case rounded bg-green-100 p-4 text-green-800">Success chip</span>
          <span className="ui-case rounded bg-yellow-100 p-4 text-yellow-800">Warning chip</span>
          <span className="ui-case rounded bg-red-100 p-4 text-red-800">Danger chip</span>
        </div>
        <div id="table-viewport" className="tailwind-card ui-case overflow-auto">
          <table id="large-table" className="w-full text-left text-sm">
            <tbody>
              {rows.map((row) => (
                <tr key={row.id} className="border-b border-slate-200">
                  <td className="px-4 py-2">{row.label}</td>
                  <td className="px-4 py-2">{row.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
