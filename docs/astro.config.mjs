// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
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
						{ label: 'DitherBadge', slug: 'components/badge' },
						{ label: 'DitherCard', slug: 'components/card' },
						{ label: 'DitherDivider', slug: 'components/divider' },
						{ label: 'DitherGradient', slug: 'components/gradient' },
						{ label: 'DitherImage', slug: 'components/image' },
						{ label: 'DitherSwitch', slug: 'components/switch' },
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
