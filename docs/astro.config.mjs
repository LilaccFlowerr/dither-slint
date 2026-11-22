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
			customCss: ['./src/styles/custom.css'],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/LilaccFlowerr/dither-slint' },
			],
			sidebar: [
				{
					label: 'Get started',
					items: [
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'Live demo', slug: 'guides/live' },
						{ label: 'Credits', slug: 'guides/credits' },
					],
				},
				{
					label: 'Components',
					items: [
						{ label: 'DitherButton', slug: 'components/button' },
						{ label: 'DitherAvatar', slug: 'components/avatar' },
						{ label: 'DitherBadge', slug: 'components/badge' },
						{ label: 'DitherCard', slug: 'components/card' },
						{ label: 'DitherDivider', slug: 'components/divider' },
						{ label: 'DitherDropdownMenu', slug: 'components/dropdown' },
						{ label: 'DitherGradient', slug: 'components/gradient' },
						{ label: 'DitherEmptyState', slug: 'components/empty-state' },
						{ label: 'DitherGauge', slug: 'components/gauge' },
						{ label: 'DitherImage', slug: 'components/image' },
						{ label: 'DitherInput', slug: 'components/input' },
						{ label: 'DitherLoader', slug: 'components/loader' },
						{ label: 'DitherProgressBar', slug: 'components/progress-bar' },
						{ label: 'DitherSkeleton', slug: 'components/skeleton' },
						{ label: 'DitherSlider', slug: 'components/slider' },
						{ label: 'DitherSparkline', slug: 'components/sparkline' },
						{ label: 'DitherStatCard', slug: 'components/stat-card' },
						{ label: 'DitherSwitch', slug: 'components/switch' },
						{ label: 'DitherTabs', slug: 'components/tabs' },
						{ label: 'DitherUptimeBar', slug: 'components/uptime-bar' },
					],
				},
				{ label: 'Motion', items: [{ label: 'Overview', slug: 'motion/overview' }] },
				{ label: 'Backgrounds', items: [{ label: 'Overview', slug: 'backgrounds/overview' }] },
				{
					label: 'Engine',
					items: [
						{ label: 'Matrices', slug: 'engine/matrices' },
						{ label: 'Palettes', slug: 'engine/palettes' },
						{ label: 'Write your own dither', slug: 'engine/write-your-own' },
					],
				},
			],
		}),
	],
});
