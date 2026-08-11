// @ts-check
import spotlightjs from "@spotlightjs/astro";
import svelte from "@astrojs/svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
	integrations: [svelte(), spotlightjs()],
	vite: {
		plugins: [tailwindcss()],
	},
});
