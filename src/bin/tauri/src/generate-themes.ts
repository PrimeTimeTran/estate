import fs from "fs";

import { darken, lighten, transparentize } from "polished";

const outlineKeys = ["outline", "outline-variant"];
export const surfacePalette = {
  // --- BASE LAYERS ---
  background: { light: "#F2F4F7", dark: "#09090b" },
  // --- ELEVATION LAYERS ---
  lowest: { light: "#ebedef", dark: "#121214" },
  surface: { light: "#ffffff", dark: "#141416" },
  low: { light: "#f8f9fa", dark: "#161618" },
  level: { light: "#f0f2f5", dark: "#1b1b1e" },
  high: { light: "#e4e6eb", dark: "#222225" },
  highest: { light: "#d8dadf", dark: "#2a2a2e" },

  // --- UI COMPONENTS ---
  surfaceVariant: { light: "#ccd0d5", dark: "#202127" },

  // --- TEXT ---
  onSurface: { light: "#18181b", dark: "#f4f4f5" },
  onSurfaceVariant: { light: "#64748b", dark: "#a8a8b2" },
};

export const intentPalette = {
  // ======================================================
  // PRIMARY
  // ======================================================
  primary: { light: "#6750a4", dark: "#d0bcff" },
  "on-primary": { light: "#ffffff", dark: "#21005d" },
  "primary-container": { light: "#eaddff", dark: "#4f378b" },
  "on-primary-container": { light: "#21005d", dark: "#eaddff" },

  // ======================================================
  // SECONDARY
  // ======================================================
  secondary: { light: "#625b71", dark: "#ccc2dc" },
  "on-secondary": { light: "#ffffff", dark: "#332d41" },
  "secondary-container": { light: "#e8def8", dark: "#4a4458" },
  "on-secondary-container": { light: "#1d192b", dark: "#e8def8" },

  // ======================================================
  // TERTIARY
  // ======================================================
  tertiary: { light: "#7d5260", dark: "#efb8c8" },
  "on-tertiary": { light: "#ffffff", dark: "#492532" },
  "tertiary-container": { light: "#ffd8e4", dark: "#633b48" },
  "on-tertiary-container": { light: "#31111d", dark: "#ffd8e4" },

  // ======================================================
  // ERROR
  // ======================================================
  error: { light: "#b3261e", dark: "#601410" },
  "on-error": { light: "#ffffff", dark: "#e7d0cf" },
  "error-container": { light: "#f9dedc", dark: "#8c1d18" },
  "on-error-container": { light: "#410e0b", dark: "#f9dedc" },

  // ======================================================
  // SUCCESS
  // ======================================================
  success: { light: "#386a20", dark: "#b7f397" },
  "on-success": { light: "#ffffff", dark: "#0f2e00" },
  "success-container": { light: "#d7f5c5", dark: "#275b1a" },
  "on-success-container": { light: "#0f2e00", dark: "#d7f5c5" },

  // ======================================================
  // WARNING
  // ======================================================
  warning: { light: "#725c0c", dark: "#f1e1a6" },
  "on-warning": { light: "#ffffff", dark: "#261a00" },
  "warning-container": { light: "#ffe9a3", dark: "#5a4300" },
  "on-warning-container": { light: "#261a00", dark: "#ffe9a3" },

  // ======================================================
  // INFO
  // ======================================================
  info: { light: "#00639b", dark: "#a8c7ff" },
  "on-info": { light: "#ffffff", dark: "#001d36" },
  "info-container": { light: "#d1e4ff", dark: "#004a77" },
  "on-info-container": { light: "#001d36", dark: "#d1e4ff" },

  // ======================================================
  // SUPPORT / NEUTRALS
  // ======================================================
  outline: { light: "#79747e", dark: "#938f99" },
  "outline-variant": { light: "#cac4d0", dark: "#49454f" },
};

const effectPalette = {
  "shadow-xs": {
    light: "0 1px 2px rgb(0 0 0 / 0.04)",
    dark: "0 1px 2px rgb(0 0 0 / 0.35)",
  },
  "shadow-sm": {
    light: "0 2px 4px rgb(0 0 0 / 0.06)",
    dark: "0 2px 4px rgb(0 0 0 / 0.40)",
  },
  "shadow-md": {
    light: "0 6px 16px rgb(0 0 0 / 0.10)",
    dark: "0 6px 16px rgb(0 0 0 / 0.45)",
  },
  "shadow-lg": {
    light: "0 12px 32px rgb(0 0 0 / 0.14)",
    dark: "0 12px 32px rgb(0 0 0 / 0.55)",
  },
  "glow-primary": {
    light: "0 0 24px rgb(103 80 164 / 0.28)",
    dark: "0 0 32px rgb(208 188 255 / 0.22)",
  },
  "glow-secondary": {
    light: "0 0 24px rgb(98 91 113 / 0.22)",
    dark: "0 0 32px rgb(204 194 220 / 0.18)",
  },
  "focus-ring": {
    light: "0 0 0 3px rgb(103 80 164 / 0.30)",
    dark: "0 0 0 3px rgb(208 188 255 / 0.24)",
  },
  glass: {
    light: "rgb(255 255 255 / 0.72)",
    dark: "rgb(255 255 255 / 0.05)",
  },
  "glass-border": {
    light: "rgb(255 255 255 / 0.65)",
    dark: "rgb(255 255 255 / 0.08)",
  },
  scrim: {
    light: "rgb(0 0 0 / 0.42)",
    dark: "rgb(0 0 0 / 0.62)",
  },
  "scrim-light": {
    light: "rgb(0 0 0 / 0.08)",
    dark: "rgb(0 0 0 / 0.22)",
  },
  "scrim-heavy": {
    light: "rgb(0 0 0 / 0.72)",
    dark: "rgb(0 0 0 / 0.82)",
  },
  "surface-tint": {
    light: "rgb(103 80 164 / 0.05)",
    dark: "rgb(208 188 255 / 0.06)",
  },
  "gradient-primary": {
    light: "linear-gradient(135deg, #7c3aed 0%, #6366f1 45%, #4338ca 100%)",
    dark: "linear-gradient(135deg, #8b5cf6 0%, #7c3aed 45%, #6366f1 100%)",
  },
};

const base = `
  body {
    @apply bg-background text-on-surface;
  }
`;

export function generateCSS() {
  const stateTransforms = {
    hover: (c: ReturnType<typeof deriveStates>) => c.hover,
    active: (c: ReturnType<typeof deriveStates>) => c.active,
    disabled: (c: ReturnType<typeof deriveStates>) => c.disabled,
  } as const;

  let themeBlock = `@theme {\n`;
  let rootBlock = `:root {\n`;
  let darkBlock = `.dark {\n`;

  // 1. Process Surfaces
  for (const [key, val] of Object.entries(surfacePalette)) {
    const token = toKebab(key);

    // raw CSS variables
    rootBlock += `  --${token}: ${val.light};\n`;
    darkBlock += `  --${token}: ${val.dark};\n`;

    // Tailwind theme tokens
    themeBlock += `  --color-${token}: var(--${token});\n`;
  }
  Object.entries(intentPalette).forEach(([key, val]) => {
    // ======================================================
    // A. BASE TOKEN
    // ======================================================
    rootBlock += `  --${key}: ${val.light};\n`;
    darkBlock += `  --${key}: ${val.dark};\n`;
    themeBlock += `  --color-${key}: var(--${key});\n`;

    // ======================================================
    // B. STATES (ONLY FOR BASE TOKENS)
    // ======================================================
    if (isStatefulToken(key)) {
      const light = deriveStates(val.light);
      const dark = deriveStates(val.dark);

      for (const state of Object.keys(stateTransforms) as Array<keyof typeof stateTransforms>) {
        const transform = stateTransforms[state];

        const lightValue = transform(light);
        const darkValue = transform(dark);

        rootBlock += `  --${key}-${state}: ${lightValue};\n`;
        darkBlock += `  --${key}-${state}: ${darkValue};\n`;
        themeBlock += `  --color-${key}-${state}: var(--${key}-${state});\n`;
      }
    }
  });

  // 3. Process Outlines (Static)
  outlineKeys.forEach((key) => {
    const varName = `--color-${key}`;
    themeBlock += `  ${varName}: var(--${key});\n`;
    rootBlock += `  ${varName}: var(--${key});\n`;
  });

  Object.entries(effectPalette).forEach(([key, val]) => {
    rootBlock += `  --${key}: ${val.light};\n`;
    darkBlock += `  --${key}: ${val.dark};\n`;
    themeBlock += `  --${key}: var(--${key});\n`;
  });

  return `@custom-variant dark (&:where(.dark, .dark *));\n\n${themeBlock}}\n\n@layer base {\n${rootBlock}}\n\n${darkBlock}}\n\n${base}\n}`;
}

function isStatefulToken(key: string) {
  return !key.startsWith("on-") && !key.includes("-container") && !key.startsWith("outline");
}
function buildDerivedIntentPalette(palette: Record<string, any>) {
  return Object.fromEntries(
    Object.entries(palette).flatMap(([key, val]) => {
      const light = deriveStates(val.light);
      const dark = deriveStates(val.dark);

      const entries: [string, any][] = [
        [
          key,
          {
            light: light.base,
            dark: dark.base,
          },
        ],
      ];

      if (isStatefulToken(key)) {
        entries.push(
          [
            `${key}-hover`,
            {
              light: light.hover,
              dark: dark.hover,
            },
          ],

          [
            `${key}-active`,
            {
              light: light.active,
              dark: dark.active,
            },
          ],

          [
            `${key}-disabled`,
            {
              light: light.disabled,
              dark: dark.disabled,
            },
          ],
        );
      }

      return entries;
    }),
  );
}
export const resolvedIntentPalette = buildDerivedIntentPalette(intentPalette);

function toKebab(str: string) {
  return str
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1-$2")
    .toLowerCase();
}

export function deriveStates(hex: string) {
  return {
    base: hex,

    // subtle interaction
    hover: lighten(0.04, hex),

    // pressed state
    active: darken(0.08, hex),

    // unavailable
    disabled: transparentize(0.45, hex),
  };
}
function main() {
  fs.writeFileSync("new.base.css", generateCSS());
  console.log("Successfully generated new.base.css using the unified intentPalette!");
}

// main();
