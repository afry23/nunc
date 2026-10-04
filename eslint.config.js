import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";
import prettier from "eslint-config-prettier";

export default tseslint.config(
  {
    ignores: [
      "dist",
      "target",
      "node_modules",
      "src-tauri",
      "crates",
      "coverage",
      "src/bindings.ts",
    ],
  },
  js.configs.recommended,
  tseslint.configs.strict,
  reactHooks.configs.flat.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
  },
  {
    // Backend-Zugriff nur über src/lib/api.ts (Typ-Bridge)
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/lib/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              importNames: ["invoke"],
              message:
                "invoke nur in src/lib/ verwenden; Commands über src/lib/api.",
            },
          ],
          patterns: [
            {
              regex: "(^|/)bindings$",
              message: "Bindings nur über src/lib/api importieren.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["*.config.{js,ts}", "scripts/**/*.mjs"],
    languageOptions: { globals: globals.node },
  },
  prettier,
);
