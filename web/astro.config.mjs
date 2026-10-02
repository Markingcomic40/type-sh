// @ts-check
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import tailwindcss from '@tailwindcss/vite';
import { readFileSync } from 'node:fs';

const cargoToml = readFileSync(
  new URL('../Cargo.toml', import.meta.url),
  'utf8',
);
const version = cargoToml.match(/^version = "(.+)"$/m)?.[1];
if (!version) throw new Error('no version found in Cargo.toml');

export default defineConfig({
  site: 'https://typesh.xyz',
  integrations: [react()],
  vite: {
    plugins: [tailwindcss()],
    define: {
      __APP_VERSION__: JSON.stringify(version),
    },
  },
});
