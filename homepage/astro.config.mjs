// @ts-check
import spotlightjs from "@spotlightjs/astro";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
	integrations: [spotlightjs()],
	vite: {
		plugins: [tailwindcss()],
	},
});
