import fsd from "@feature-sliced/steiger-plugin";
import { defineConfig } from "steiger";

export default defineConfig([
  ...fsd.configs.recommended,
  // shadcn CLI output: imported per file by the design system, never through an index.
  { ignores: ["./src/shared/ui/primitives/**"] },
]);
