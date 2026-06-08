// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures.md#component
// CODEGEN-BEGIN
import React from "react";
import ReactDOM from "react-dom/client";
import Button from "@mui/material/Button";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import "./style.css";

const cjsMessage = require("./message.cjs");

const theme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#315efb",
    },
    secondary: {
      main: "#0a7f68",
    },
  },
});

function ProductionBuildFixture() {
  const [count, setCount] = React.useState(0);
  const message = cjsMessage.exports?.message ?? cjsMessage.message ?? "missing cjs";

  return (
    <ThemeProvider theme={theme}>
      <main className="production-fixture">
        <p className="eyebrow">Jet production regression fixture</p>
        <h1>Jet production regression fixture</h1>
        <p data-testid="cjs-message">{message}</p>
        <Button
          variant="contained"
          color="primary"
          data-testid="mui-button"
          onClick={() => setCount((value) => value + 1)}
        >
          MUI boot count {count}
        </Button>
      </main>
    </ThemeProvider>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ProductionBuildFixture />
  </React.StrictMode>,
);
// CODEGEN-END
