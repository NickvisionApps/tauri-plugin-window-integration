import js from "@eslint/js";
import {defineConfig, globalIgnores} from "eslint/config";
import eslintConfigPrettier from "eslint-config-prettier/flat";
import simpleImportSort from "eslint-plugin-simple-import-sort";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["node_modules", "dist-js"]),
  {
    files: ["src/**/*.ts"],
    extends: [js.configs.recommended, tseslint.configs.strictTypeChecked],
    plugins: {
      "simple-import-sort": simpleImportSort,
    },
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "simple-import-sort/imports": [
        "error",
        {
          groups: [["^\\u0000"], ["^node:"], ["^@?\\w"], ["^@/"], ["^\\."]],
        },
      ],
      "simple-import-sort/exports": "error",
    },
  },
  {
    files: ["eslint.config.js", "rollup.config.js"],
    extends: [js.configs.recommended],
    plugins: {
      "simple-import-sort": simpleImportSort,
    },
    languageOptions: {
      globals: {
        URL: "readonly",
      },
    },
    rules: {
      "simple-import-sort/imports": [
        "error",
        {
          groups: [["^\\u0000"], ["^node:"], ["^@?\\w"], ["^@/"], ["^\\."]],
        },
      ],
      "simple-import-sort/exports": "error",
    },
  },
  eslintConfigPrettier,
]);
