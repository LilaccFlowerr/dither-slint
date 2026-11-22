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
							label: 'Basics',
							items: [
								{ label: 'DitherButton', slug: 'components/button' },
								{ label: 'DitherBadge', slug: 'components/badge' },
								{ label: 'DitherCard', slug: 'components/card' },
								{ label: 'DitherDivider', slug: 'components/divider' },
								{ label: 'DitherGradient', slug: 'components/gradient' },
							],
						},
						{
							label: 'Inputs',
							items: [
								{ label: 'DitherSwitch', slug: 'components/switch' },
								{ label: 'DitherSlider', slug: 'components/slider' },
								{ label: 'DitherInput', slug: 'components/input' },
								{ label: 'DitherTabs', slug: 'components/tabs' },
								{ label: 'DitherDropdownMenu', slug: 'components/dropdown' },
							],
						},
						{
							label: 'Data',
							items: [
								{ label: 'DitherLineChart', slug: 'components/line-chart' },
								{ label: 'DitherAreaChart', slug: 'components/area-chart' },
								{ label: 'DitherBarChart', slug: 'components/bar-chart' },
								{ label: 'DitherDonutChart', slug: 'components/donut-chart' },
								{ label: 'DitherRadarChart', slug: 'components/radar-chart' },
								{ label: 'DitherHeatmap', slug: 'components/heatmap' },
								{ label: 'DitherPixelArea', slug: 'components/pixel-area' },
								{ label: 'DitherTreemap', slug: 'components/treemap' },
								{ label: 'DitherSparkline', slug: 'components/sparkline' },
								{ label: 'DitherStatCard', slug: 'components/stat-card' },
								{ label: 'DitherUptimeBar', slug: 'components/uptime-bar' },
								{ label: 'DitherGauge', slug: 'components/gauge' },
								{ label: 'DitherNumber', slug: 'components/number' },
								{ label: 'DitherProgressBar', slug: 'components/progress-bar' },
							],
						},
						{
							label: 'Feedback and identity',
							items: [
								{ label: 'DitherAvatar', slug: 'components/avatar' },
								{ label: 'DitherLoader', slug: 'components/loader' },
								{ label: 'DitherSkeleton', slug: 'components/skeleton' },
								{ label: 'DitherEmptyState', slug: 'components/empty-state' },
							],
						},
						{
							label: 'Images',
							items: [
								{ label: 'DitherImage', slug: 'components/image' },
								{ label: 'DitherImageCompare', slug: 'components/image-compare' },
								{ label: 'DitherScrollReveal', slug: 'components/scroll-reveal' },
								{ label: 'DitherLogo', slug: 'components/logo' },
								{ label: 'DitherLogoMarquee', slug: 'components/logo-marquee' },
							],
						},
						{
							label: 'Text',
							items: [
								{ label: 'DitherText', slug: 'components/dither-text' },
								{ label: 'DitherDissolveText', slug: 'components/dissolve-text' },
								{ label: 'DitherScrambleText', slug: 'components/scramble-text' },
							],
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
