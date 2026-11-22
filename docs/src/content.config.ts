import { defineCollection, z } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

const background = z.object({
	field: z.string(),
	params: z.array(z.number()),
	palette: z.array(z.string()),
	matrix: z.number().optional(),
	cell: z.number().optional(),
	still: z.number().optional(),
	slots: z.enum(['palette', 'noise']).optional(),
	colors: z.array(z.string()).optional(),
	timeSpeed: z.boolean().optional(),
});

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema({ extend: z.object({ background: background.optional() }) }) }),
};
