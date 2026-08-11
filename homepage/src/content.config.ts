import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

const services = defineCollection({
	loader: glob({ pattern: "**/*.md", base: "./src/content/services" }),
	schema: z.object({
		title: z.string(),
		description: z.string(),
		lead: z.string(),
		icon: z.enum(["agent", "desktop", "assistant", "server", "router", "learn"]),
		highlights: z
			.array(
				z.object({
					title: z.string(),
					description: z.string(),
				}),
			)
			.optional(),
	}),
});

const home = defineCollection({
	loader: glob({ pattern: "**/*.md", base: "./src/content/home" }),
	schema: z.object({
		title: z.string(),
		description: z.string(),
		hero: z.object({
			eyebrow: z.string(),
			title: z.string(),
			lead: z.string(),
			watchDemoBadge: z.string(),
			architectureAlt: z.string(),
		}),
		why: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
			reasons: z.array(
				z.object({
					eyebrow: z.string(),
					title: z.string(),
					description: z.string(),
				}),
			),
		}),
		how: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
			features: z.array(
				z.object({
					icon: z.enum(["graph", "memory", "sdk", "markdown"]),
					eyebrow: z.string(),
					title: z.string(),
					description: z.string(),
				}),
			),
		}),
		pillars: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
			cards: z.object({
				agent: z.object({ description: z.string(), linkText: z.string() }),
				desktop: z.object({ description: z.string(), linkText: z.string() }),
				assistant: z.object({ description: z.string(), linkText: z.string() }),
				server: z.object({ description: z.string(), linkText: z.string() }),
				router: z.object({ description: z.string(), linkText: z.string() }),
				learn: z.object({ description: z.string(), linkText: z.string() }),
			}),
		}),
		developer: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
			features: z.array(
				z.object({
					eyebrow: z.string(),
					title: z.string(),
					description: z.string(),
				}),
			),
			privacyBanner: z.string(),
		}),
		waitlist: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
		}),
		license: z.object({
			eyebrow: z.string(),
			title: z.string(),
			description: z.string(),
			points: z.array(
				z.object({
					title: z.string(),
					description: z.string(),
					badge: z.string(),
				}),
			),
			note: z.string(),
		}),
	}),
});

export const collections = {
	services,
	home,
};
