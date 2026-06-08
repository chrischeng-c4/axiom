const path = require("path");

class EmitHtmlPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("EmitHtmlPlugin", (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: "EmitHtmlPlugin",
          stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const html = [
            '<!doctype html>',
            '<html lang="en">',
            '<head>',
            '  <meta charset="UTF-8" />',
            '  <meta name="viewport" content="width=device-width, initial-scale=1.0" />',
            '  <title>React Bench</title>',
            '</head>',
            '<body>',
            '  <div id="root"></div>',
            '  <script src="./main.js"></script>',
            '</body>',
            '</html>',
            '',
          ].join("\n");
          compilation.emitAsset("index.html", new compiler.webpack.sources.RawSource(html));
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
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js", ".jsx"],
  },
  module: {
    rules: [
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
  plugins: [new EmitHtmlPlugin()],
};
