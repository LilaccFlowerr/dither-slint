// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
	site: process.env.DOCS_SITE,
	base: process.env.DOCS_BASE ?? '/',
	integrations: [
		starlight({
			title: 'dither-slint',
			description: 'Dithered UI components for Slint, written in Rust.',
			expressiveCode: { shiki: { langAlias: { slint: 'qml' } } },
			customCss: ['./src/styles/custom.css'],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/LilaccFlowerr/dither-slint' },
			],
			sidebar: [
				{
					label: 'Get started',
					items: [
						{ label: 'Introduction', slug: 'guides/introduction' },
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'Credits', slug: 'guides/credits' },
					],
				},
				{
					label: 'Components',
					items: [
						{ label: 'Overview', slug: 'components' },
						{
							label: 'Interface',
							items: ['button', 'badge', 'avatar', 'card', 'divider', 'dropdown', 'empty-state', 'gradient', 'input', 'loader', 'skeleton', 'slider', 'switch', 'tabs'].map((slug) => ({ slug: `components/${slug}` })),
						},
						{
							label: 'Charts and data',
							items: ['line-chart', 'area-chart', 'bar-chart', 'donut-chart', 'radar-chart', 'heatmap', 'pixel-area', 'treemap', 'sparkline', 'gauge', 'progress-bar', 'stat-card', 'uptime-bar'].map((slug) => ({ slug: `components/${slug}` })),
						},
						{
							label: 'Images',
							items: ['image', 'image-compare', 'scroll-reveal', 'logo', 'logo-marquee'].map((slug) => ({ slug: `components/${slug}` })),
						},
						{
							label: 'Text',
							items: ['dither-text', 'dissolve-text', 'scramble-text', 'number'].map((slug) => ({ slug: `components/${slug}` })),
						},
					],
				},
				{ label: 'Motion', items: [{ label: 'Overview', slug: 'motion/overview' }] },
				{
					label: 'Backgrounds',
					collapsed: true,
					items: [
						{ label: 'Overview', slug: 'backgrounds/overview' },
						{
							label: 'Gradients',
							collapsed: true,
							items: [
								{ label: 'DitherAnimatedGradient', slug: 'backgrounds/animated-gradient' },
								{ label: 'DitherMeshGradient', slug: 'backgrounds/mesh-gradient' },
								{ label: 'DitherSilk', slug: 'backgrounds/silk' },
								{ label: 'DitherSmoke', slug: 'backgrounds/smoke' },
								{ label: 'DitherPlasma', slug: 'backgrounds/plasma' },
							],
						},
						{
							label: 'Light',
							collapsed: true,
							items: [
								{ label: 'DitherAurora', slug: 'backgrounds/aurora' },
								{ label: 'DitherBeams', slug: 'backgrounds/beams' },
								{ label: 'DitherLightRays', slug: 'backgrounds/light-rays' },
								{ label: 'DitherSpotlight', slug: 'backgrounds/spotlight' },
								{ label: 'DitherEclipse', slug: 'backgrounds/eclipse' },
								{ label: 'DitherOrb', slug: 'backgrounds/orb' },
								{ label: 'DitherHorizon', slug: 'backgrounds/horizon' },
								{ label: 'DitherStarfield', slug: 'backgrounds/starfield' },
							],
						},
						{
							label: 'Waves and noise',
							collapsed: true,
							items: [
								{ label: 'DitherRipples', slug: 'backgrounds/ripples' },
								{ label: 'DitherWaves', slug: 'backgrounds/waves' },
								{ label: 'DitherNoiseField', slug: 'backgrounds/noise-field' },
								{ label: 'DitherContours', slug: 'backgrounds/contours' },
								{ label: 'DitherMetaballs', slug: 'backgrounds/metaballs' },
								{ label: 'DitherRidgelines', slug: 'backgrounds/ridgelines' },
								{ label: 'DitherHalftone', slug: 'backgrounds/halftone' },
								{ label: 'DitherFade', slug: 'backgrounds/fade' },
							],
						},
						{
							label: 'Shapes and maps',
							collapsed: true,
							items: [
								{ label: 'DitherGlobe', slug: 'backgrounds/globe' },
								{ label: 'DitherDotMap', slug: 'backgrounds/dot-map' },
								{ label: 'DitherTunnel', slug: 'backgrounds/tunnel' },
								{ label: 'DitherRetroGrid', slug: 'backgrounds/retro-grid' },
							],
						},
					],
				},
				{
					label: 'Engine',
					items: [
						{ label: 'Matrices', slug: 'engine/matrices' },
						{ label: 'Palettes', slug: 'engine/palettes' },
						{ label: 'Write your own dither', slug: 'engine/write-your-own' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Types', slug: 'reference/types' },
						{ label: 'Theme', slug: 'reference/theme' },
					],
				},
			],
		}),
	],
});
