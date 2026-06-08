const fs = require("fs");
const path = require("path");

class EmitFixtureAssetsPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("EmitFixtureAssetsPlugin", (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: "EmitFixtureAssetsPlugin",
          stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const css = fs.readFileSync(path.resolve(__dirname, "src/main.css"));
          const brand = fs.readFileSync(path.resolve(__dirname, "public/brand.svg"));
          const html = [
            '<!doctype html>',
            '<html lang="en">',
            '<head>',
            '  <meta charset="UTF-8" />',
            '  <meta name="viewport" content="width=device-width, initial-scale=1.0" />',
            '  <title>DOM Production Assets</title>',
            '  <link rel="stylesheet" href="./main.css" />',
            '</head>',
            '<body>',
            '  <div id="root"></div>',
            '  <script src="./main.js"></script>',
            '</body>',
            '</html>',
            '',
          ].join("\n");
          compilation.emitAsset("index.html", new compiler.webpack.sources.RawSource(html));
          compilation.emitAsset("main.css", new compiler.webpack.sources.RawSource(css));
          compilation.emitAsset("brand.svg", new compiler.webpack.sources.RawSource(brand));
        },
      );
    });
  }
}

module.exports = {
  mode: "production",
  entry: "./src/main.tsx",
  output: {
    path: path.resolve(__dirname, "dist-webpack"),
    filename: "main.js",
    clean: true,
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js", ".jsx"],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [path.resolve(__dirname, "css-inject-loader.cjs")],
      },
      {
        test: /\.tsx?$/,
        use: {
          loader: "babel-loader",
          options: {
            presets: [
              "@babel/preset-env",
              ["@babel/preset-react", { runtime: "automatic" }],
              "@babel/preset-typescript",
            ],
          },
        },
        exclude: /node_modules/,
      },
    ],
  },
  plugins: [new EmitFixtureAssetsPlugin()],
};
