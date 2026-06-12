import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const siteBasePath = process.env.SCAFFOLD_SITE_BASE_PATH;
const base = siteBasePath ? `${siteBasePath.replace(/\/$/, "")}/` : "/";

export default defineConfig({
  base,
  plugins: [tailwindcss(), svelte()],
});
