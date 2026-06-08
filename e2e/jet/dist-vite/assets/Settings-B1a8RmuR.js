import { u as useState, a as useEffect, c as createElement } from "./index-fWhMswjv.js";
function Settings() {
  const [theme, setTheme] = useState("light");
  const [mounted, setMounted] = useState(false);
  useEffect(() => {
    setMounted((_) => true);
  }, []);
  return /* @__PURE__ */ createElement("div", { className: "settings-page", "data-testid": "settings-page" }, /* @__PURE__ */ createElement("h3", null, "Settings"), /* @__PURE__ */ createElement("div", { "data-testid": "settings-mounted" }, mounted ? "ready" : "loading"), /* @__PURE__ */ createElement("div", { "data-testid": "settings-theme" }, /* @__PURE__ */ createElement("span", null, "Theme: ", theme), /* @__PURE__ */ createElement(
    "button",
    {
      "data-testid": "toggle-theme",
      onClick: () => setTheme((t) => t === "light" ? "dark" : "light")
    },
    "Toggle"
  )));
}
export {
  Settings as default
};
