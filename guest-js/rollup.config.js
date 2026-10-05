import {readFileSync} from "node:fs";
import {dirname} from "node:path";

import typescript from "@rollup/plugin-typescript";

const pkg = JSON.parse(
  readFileSync(new URL("./package.json", import.meta.url)),
);

export default {
  input: "src/index.ts",
  output: [
    {file: pkg.exports.import, format: "esm"},
    {file: pkg.exports.require, format: "cjs"},
  ],
  plugins: [
    typescript({
      declaration: true,
      declarationDir: dirname(pkg.exports.import),
    }),
  ],
  external: [/^@tauri-apps\//],
};
