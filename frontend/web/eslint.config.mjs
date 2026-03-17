import { defineConfig } from "eslint/config";
import jest from "eslint-plugin-jest";
import nextCoreWebVitals from "eslint-config-next/core-web-vitals";
import nextTypeScript from "eslint-config-next/typescript";

export default defineConfig([
    ...nextCoreWebVitals,
    ...nextTypeScript,
    {
        ...jest.configs["flat/recommended"],
        files: ["**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    },
]);
