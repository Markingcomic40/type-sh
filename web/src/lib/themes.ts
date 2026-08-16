// Same palettes as the TUI, just for the web. Keys have to match the --c-* in
// global.css or the token silently doesnt get set

export type Scheme = {
  label: string;
  bg: string;
  bg2: string;
  termBg: string;
  fg: string;
  fg2: string;
  muted: string;
  primary: string;
  secondary: string;
  correct: string;
  wrong: string;
  wrongSubtle: string;
  border: string;
  prompt: string;
  path: string;
  info: string;
  warn: string;
  err: string;
  accent: string;
};

export const SCHEMES: Record<string, Scheme> = {
  gruvbox: {
    label: "Gruvbox",
    bg: "#1d2021",
    bg2: "#282828",
    termBg: "#1d2021",
    fg: "#ebdbb2",
    fg2: "#d5c4a1",
    muted: "#7c6f64",
    primary: "#fabd2f",
    secondary: "#928374",
    correct: "#b8bb26",
    wrong: "#fb4934",
    wrongSubtle: "#cc241d",
    border: "#3c3836",
    prompt: "#8ec07c",
    path: "#83a598",
    info: "#83a598",
    warn: "#fabd2f",
    err: "#fb4934",
    accent: "#d3869b",
  },
  solarized: {
    label: "Solarized Dark",
    bg: "#002b36",
    bg2: "#073642",
    termBg: "#002b36",
    fg: "#fdf6e3",
    fg2: "#eee8d5",
    muted: "#586e75",
    primary: "#b58900",
    secondary: "#657b83",
    correct: "#859900",
    wrong: "#dc322f",
    wrongSubtle: "#cb4b16",
    border: "#073642",
    prompt: "#2aa198",
    path: "#268bd2",
    info: "#268bd2",
    warn: "#b58900",
    err: "#dc322f",
    accent: "#d33682",
  },
};

export const DEFAULT_SCHEME = "gruvbox";

// Client side only, touches document
export function applyScheme(name: string): void {
  const s = SCHEMES[name] ?? SCHEMES[DEFAULT_SCHEME];
  const root = document.documentElement;
  for (const [k, v] of Object.entries(s)) {
    if (k === "label") continue;
    root.style.setProperty(`--c-${k}`, v);
  }
}
