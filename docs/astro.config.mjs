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
						{ label: 'DitherAvatar', slug: 'components/avatar' },
						{ label: 'DitherBadge', slug: 'components/badge' },
						{ label: 'DitherCard', slug: 'components/card' },
						{ label: 'DitherDivider', slug: 'components/divider' },
						{ label: 'DitherDropdownMenu', slug: 'components/dropdown' },
						{ label: 'DitherGradient', slug: 'components/gradient' },
						{ label: 'DitherImage', slug: 'components/image' },
						{ label: 'DitherInput', slug: 'components/input' },
						{ label: 'DitherLoader', slug: 'components/loader' },
						{ label: 'DitherSlider', slug: 'components/slider' },
						{ label: 'DitherSwitch', slug: 'components/switch' },
						{ label: 'DitherTabs', slug: 'components/tabs' },
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
