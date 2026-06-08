module.exports = function cssInjectLoader(source) {
  const css = JSON.stringify(source);
  return `
    const style = document.createElement("style");
    style.setAttribute("data-webpack-fixture-css", "dom-production-assets");
    style.textContent = ${css};
    document.head.appendChild(style);
  `;
};
