import React, { useState } from "react";
import Alert from "antd/es/alert";
import Avatar from "antd/es/avatar";
import Badge from "antd/es/badge";
import Breadcrumb from "antd/es/breadcrumb";
import Button from "antd/es/button";
import Card from "antd/es/card";
import Checkbox from "antd/es/checkbox";
import Col from "antd/es/col";
import Divider from "antd/es/divider";
import Flex from "antd/es/flex";
import Form from "antd/es/form";
import Input from "antd/es/input";
import InputNumber from "antd/es/input-number";
import List from "antd/es/list";
import Menu from "antd/es/menu";
import Pagination from "antd/es/pagination";
import Progress from "antd/es/progress";
import Radio from "antd/es/radio";
import Rate from "antd/es/rate";
import Row from "antd/es/row";
import Select from "antd/es/select";
import Slider from "antd/es/slider";
import Space from "antd/es/space";
import Spin from "antd/es/spin";
import Steps from "antd/es/steps";
import Switch from "antd/es/switch";
import Table from "antd/es/table";
import Tabs from "antd/es/tabs";
import Tag from "antd/es/tag";
import Timeline from "antd/es/timeline";
import Tooltip from "antd/es/tooltip";

export function AntdVisualFixture() {
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
        <h1 style={{ margin: 0 }}>AntD visual table fixture</h1>
        <p style={{ margin: "4px 0 0" }}>
          AntD component matrix and large-table workload
        </p>
      </header>

      <section aria-label="AntD large table workload">
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
        id="antd-component-matrix"
        className="component-matrix"
        aria-label="AntD component matrix"
        style={{ marginTop: 24 }}
      >
        <h2>AntD component matrix</h2>
        <Space direction="vertical" size="middle" style={{ display: "flex" }}>
          <Row gutter={16} className="ui-case antd-grid">
            <Col span={12}>
              <Card title="Card surface" className="ui-case antd-card">
                <p>hello {name}; accepted: {String(accepted)}</p>
                <Space wrap={true} className="ui-case antd-actions">
                  <Button id="antd-button" type="primary" className="ui-case antd-button-primary">
                    AntD Primary
                  </Button>
                  <Button className="ui-case antd-button-default">Default</Button>
                  <Tooltip title="Tooltip content">
                    <Button className="ui-case antd-tooltip-button">Tooltip</Button>
                  </Tooltip>
                </Space>
              </Card>
            </Col>
            <Col span={12}>
              <Card title="Form controls" className="ui-case antd-form-controls">
                <Form layout="vertical">
                  <label htmlFor="antd-name">Name</label>
                  <div style={{ marginBottom: 12 }}>
                    <Input
                      id="antd-name"
                      className="ui-case antd-input"
                      value={name}
                      onChange={(event) => setName(event.target.value)}
                    />
                  </div>
                  <label>Number</label>
                  <div style={{ marginBottom: 12 }}>
                    <InputNumber className="ui-case antd-input-number" value={42} />
                  </div>
                  <Checkbox
                    id="antd-accept"
                    className="ui-case antd-checkbox"
                    checked={accepted}
                    onChange={(event) => setAccepted(event.target.checked)}
                  >
                    Accept library terms
                  </Checkbox>
                  <Switch className="ui-case antd-switch" checked={enabled} />
                </Form>
              </Card>
            </Col>
          </Row>

          <Flex gap="middle" wrap="wrap" className="ui-case antd-feedback">
            <Alert
              className="ui-case antd-alert"
              message="AntD success alert"
              type="success"
              showIcon={true}
            />
            <Progress className="ui-case antd-progress" percent={64} />
            <Spin className="ui-case antd-spin" />
            <Tag className="ui-case antd-tag" color="blue">
              Tag
            </Tag>
            <Badge className="ui-case antd-badge" count={4}>
              <Avatar>A</Avatar>
            </Badge>
          </Flex>

          <Flex gap="middle" wrap="wrap" className="ui-case antd-inputs">
            <Radio className="ui-case antd-radio" checked={true}>
              Radio
            </Radio>
            <Rate className="ui-case antd-rate" value={3} />
            <Slider className="ui-case antd-slider" value={36} style={{ width: 180 }} />
            <Select
              className="ui-case antd-select"
              defaultValue="balanced"
              style={{ width: 160 }}
            />
          </Flex>

          <Divider className="ui-case antd-divider">Navigation</Divider>
          <Flex gap="middle" wrap="wrap" className="ui-case antd-navigation">
            <Breadcrumb className="ui-case antd-breadcrumb" />
            <Menu className="ui-case antd-menu" mode="horizontal" />
            <Pagination className="ui-case antd-pagination" current={2} total={80} />
            <Steps className="ui-case antd-steps" current={1} />
            <Tabs className="ui-case antd-tabs" defaultActiveKey="1" />
          </Flex>

          <Flex gap="middle" wrap="wrap" className="ui-case antd-data-display">
            <List className="ui-case antd-list" />
            <Table className="ui-case antd-table" pagination={false} />
            <Timeline className="ui-case antd-timeline" />
          </Flex>
        </Space>
      </section>
    </main>
  );
}
