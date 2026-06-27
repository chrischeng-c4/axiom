// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-styled-components-visual-src-main-tsx" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import styled, { createGlobalStyle, css } from "styled-components";

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
  libraryId: "styled-components",
  tableTitle: "styled-components visual table fixture",
  minComponentCases: 8,
  primaryButtonText: "Styled Primary",
};

const GlobalStyle = createGlobalStyle`
  body {
    margin: 0;
    background: #f4f7fb;
    color: #152033;
    font-family: Inter, ui-sans-serif, system-ui, sans-serif;
  }
`;

const Matrix = styled.main`
  min-height: 100vh;
  padding: 32px;
`;

const Surface = styled.section`
  display: grid;
  gap: 16px;
`;

const Card = styled.div`
  background: #ffffff;
  border: 1px solid #d7dee9;
  border-radius: 8px;
  box-shadow: 0 10px 30px rgba(31, 41, 55, 0.08);
  padding: 18px;
`;

const Button = styled.button`
  border: 0;
  border-radius: 6px;
  color: #ffffff;
  cursor: pointer;
  min-height: 36px;
  padding: 0 14px;
  ${(props) => css`
    background: ${props.$accent || "#2563eb"};
  `}
`;

const Chip = styled.span`
  background: ${(props) => props.$tone || "#eef2ff"};
  border-radius: 999px;
  display: inline-flex;
  padding: 8px 12px;
`;

const TableViewport = styled.div`
  max-height: 220px;
  overflow: auto;
`;

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
    <>
      <GlobalStyle />
      <Matrix>
        <Surface className="component-matrix">
          <Card className="ui-case">
            <p>styled-components visual table fixture</p>
            <h1>styled-components component matrix</h1>
            <p>Runtime CSS-in-JS with tagged templates and dynamic props</p>
          </Card>
          <Card className="ui-case">
            <Button
              type="button"
              className="ui-case"
              $accent="#2563eb"
              onClick={() => setActive((value) => value + 1)}
            >
              Styled Primary
            </Button>
            <Chip className="ui-case" $tone="#dcfce7">
              Styled count {active}
            </Chip>
          </Card>
          <Card className="ui-case">
            <Chip className="ui-case" $tone="#dbeafe">Info chip</Chip>
            <Chip className="ui-case" $tone="#fef3c7">Warning chip</Chip>
            <Chip className="ui-case" $tone="#fee2e2">Danger chip</Chip>
          </Card>
          <Card className="ui-case">
            <TableViewport id="table-viewport">
              <table id="large-table">
                <tbody>
                  {rows.map((row) => (
                    <tr key={row.id}>
                      <td>{row.label}</td>
                      <td>{row.status}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </TableViewport>
          </Card>
        </Surface>
      </Matrix>
    </>
  );
}

createRoot(document.getElementById("root")!).render(<App />);

// </HANDWRITE>
