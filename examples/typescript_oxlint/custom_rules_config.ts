import { createExampleParserOptions } from "./shared.ts";
import { typescriptOxlintCustomPlugin } from "./custom_plugin.ts";

const config = [
  {
    settings: {
      typescriptOxlint: {
        parserOptions: createExampleParserOptions(),
      },
    },
    plugins: {
      example: typescriptOxlintCustomPlugin,
    },
    rules: {
      "example/no-string-plus-number": "error",
    },
  },
];

export default config;
