// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-data-fixtures-mui-mui-checkbox-basic-v1.md#component
// CODEGEN-BEGIN
import * as React from "react";
import Checkbox from "@mui/material/Checkbox";

export default function MuiCheckboxBasicV1(): JSX.Element {
  return (
    <Checkbox defaultChecked inputProps={{ "aria-label": "basic checkbox" }} />
  );
}
// CODEGEN-END
