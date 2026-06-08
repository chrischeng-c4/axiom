import React, { useState } from "react";
import Button from "@mui/material/Button";
import Checkbox from "@mui/material/Checkbox";
import TextField from "@mui/material/TextField";

export function MuiVisualFixture() {
  const [name, setName] = useState("Ada");
  const [accepted, setAccepted] = useState(true);

  return (
    <main
      id="visual-root"
      style={{
        fontFamily: "Inter, system-ui, sans-serif",
        maxWidth: 440,
        margin: "32px auto",
        padding: 24,
        border: "1px solid #d7dde8",
        borderRadius: 8,
      }}
    >
      <h1 style={{ fontSize: 24, margin: "0 0 16px" }}>MUI visual fixture</h1>
      <TextField
        id="mui-name"
        label="Name"
        value={name}
        onChange={(event) => setName(event.target.value)}
        fullWidth
        margin="normal"
      />
      <label
        htmlFor="mui-accept"
        style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 12 }}
      >
        <Checkbox
          id="mui-accept"
          checked={accepted}
          onChange={(event) => setAccepted(event.target.checked)}
        />
        Accept library terms
      </label>
      <Button id="mui-button" variant="contained" color="primary" sx={{ mt: 2 }}>
        MUI Primary
      </Button>
      <p id="echo" style={{ marginTop: 18 }}>
        hello {name}; accepted: {String(accepted)}
      </p>
    </main>
  );
}
