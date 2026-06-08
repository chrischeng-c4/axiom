// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-fixtures.md#component
// CODEGEN-BEGIN
/** @fixture { "name": "mui-button", "ime": false, "tab_count": 8 } */
import * as React from "react";
import Button from "@mui/material/Button";

export default function MuiButton() {
  return (
    <Button variant="contained" data-jet-fixture="mui-button">
      Click me
    </Button>
  );
}
// CODEGEN-END
