// website/uno.config.ts
import { defineConfig, presetUno, presetTypography } from "unocss";

export default defineConfig({
  presets: [
    presetUno(),
    presetTypography(),
  ],
  theme: {
    colors: {
      gray: {
        100: "#f3f4f6",
        400: "#9ca3af",
        700: "#374151",
        800: "#1f2937",
        900: "#111827",
      },
      cyan: {
        400: "#22d3ee",
        500: "#06b6d4",
        600: "#0891b2",
      },
      green: {
        600: "#16a34a",
      },
    },
  },
});
