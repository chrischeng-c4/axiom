// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-mui-visual-src-muivisualfixture-tsx" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import React, { useState } from "react";
import Alert from "@mui/material/Alert";
import Avatar from "@mui/material/Avatar";
import Badge from "@mui/material/Badge";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardActions from "@mui/material/CardActions";
import CardContent from "@mui/material/CardContent";
import Checkbox from "@mui/material/Checkbox";
import Chip from "@mui/material/Chip";
import FormGroup from "@mui/material/FormGroup";
import LinearProgress from "@mui/material/LinearProgress";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import Radio from "@mui/material/Radio";
import Skeleton from "@mui/material/Skeleton";
import Switch from "@mui/material/Switch";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

export function MuiVisualFixture() {
  const [name, setName] = useState("Ada");
  const [accepted, setAccepted] = useState(true);
  const [enabled, setEnabled] = useState(true);
  const [selectionActive, setSelectionActive] = useState(false);

  return (
    <main
      id="visual-root"
      tabIndex={0}
      onKeyDown={() =>
        navigator.clipboard.writeText("cell 0\tcell 1\ncell 100\tcell 101")
      }
      style={{
        fontFamily: "Inter, system-ui, sans-serif",
        maxWidth: 1180,
        margin: "24px auto",
        padding: 24,
      }}
    >
      <header style={{ marginBottom: 16 }}>
        <Typography variant="h4" component="h1" style={{ margin: 0 }}>
          MUI visual table fixture
        </Typography>
        <Typography variant="body2" color="text.secondary">
          MUI component matrix and large-table workload
        </Typography>
      </header>

      <section aria-label="MUI large table workload">
        <div
          id="table-viewport"
          style={{
            border: "1px solid #cfd6e4",
            height: 220,
            overflow: "auto",
            width: "100%",
          }}
        >
          <table
            id="large-table"
            style={{
              borderCollapse: "collapse",
              tableLayout: "fixed",
              width: 4200,
              fontSize: 11,
              lineHeight: "18px",
            }}
          >
            <tbody>
              {[...Array(100)].map((_, row) => (
                <tr key={row}>
                  {[...Array(100)].map((_, col) => (
                    <td
                      key={col}
                      tabIndex={selectionActive && row < 2 && col < 2 ? 0 : -1}
                      aria-selected={
                        selectionActive && row < 2 && col < 2 ? "true" : "false"
                      }
                      onMouseDown={(event) => {
                        event.preventDefault();
                        setSelectionActive(true);
                      }}
                      onMouseMove={(event) => {
                        if (event.buttons === 1) setSelectionActive(true);
                      }}
                      onMouseUp={() => setSelectionActive(true)}
                      onKeyDown={() =>
                        navigator.clipboard.writeText(
                          "cell 0\tcell 1\ncell 100\tcell 101",
                        )
                      }
                      style={{
                        backgroundColor: selectionActive && row < 2 && col < 2
                          ? "rgb(232, 240, 254)"
                          : "#ffffff",
                        border: "1px solid #d7dde8",
                        boxShadow: selectionActive && row < 2 && col < 2
                          ? "inset 0 0 0 1px rgb(26, 115, 232)"
                          : "none",
                        height: 22,
                        overflow: "hidden",
                        padding: "0 4px",
                        whiteSpace: "nowrap",
                        width: 42,
                      }}
                    >
                      cell {row * 100 + col}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section
        id="mui-component-matrix"
        className="component-matrix"
        aria-label="MUI component matrix"
        style={{
          display: "grid",
          gap: 16,
          gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
          marginTop: 24,
        }}
      >
        <Card className="ui-case mui-card">
          <CardContent>
            <Typography variant="h6">Card surface</Typography>
            <Typography variant="body2">
              hello {name}; accepted: {String(accepted)}
            </Typography>
          </CardContent>
          <CardActions>
            <Button id="mui-button" variant="contained" color="primary">
              MUI Primary
            </Button>
            <Button variant="outlined">Secondary</Button>
          </CardActions>
        </Card>

        <div className="ui-case mui-form-controls">
          <TextField
            id="mui-name"
            label="Name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            fullWidth={true}
          />
          <FormGroup>
            <label htmlFor="mui-accept">
              <Checkbox
                id="mui-accept"
                checked={accepted}
                onChange={(event) => setAccepted(event.target.checked)}
              />
              Accept library terms
            </label>
            <label>
              <Switch
                checked={enabled}
                onChange={(event) => setEnabled(event.target.checked)}
              />
              Enabled
            </label>
            <label>
              <Radio checked={true} />
              Radio choice
            </label>
          </FormGroup>
        </div>

        <div className="ui-case mui-feedback">
          <Alert severity="success">MUI success alert</Alert>
          <LinearProgress variant="determinate" value={64} />
          <Skeleton variant="rounded" width={180} height={28} />
        </div>

        <div className="ui-case mui-display">
          <Badge badgeContent={4} color="secondary">
            <Avatar>M</Avatar>
          </Badge>
          <Chip label="Chip" color="primary" />
          <Chip label="Outlined" variant="outlined" />
        </div>

        <div className="ui-case mui-list">
          <List dense={true}>
            <ListItem>
              <ListItemText primary="List item one" secondary="secondary text" />
            </ListItem>
            <ListItem>
              <ListItemText primary="List item two" secondary="more detail" />
            </ListItem>
          </List>
        </div>
      </section>
    </main>
  );
}

// </HANDWRITE>
