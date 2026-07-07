// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-dom-production-assets-css-inject-loader-cjs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
module.exports = function cssInjectLoader(source) {
  const css = JSON.stringify(source);
  return `
    const style = document.createElement("style");
    style.setAttribute("data-webpack-fixture-css", "dom-production-assets");
    style.textContent = ${css};
    document.head.appendChild(style);
  `;
};

// </HANDWRITE>
