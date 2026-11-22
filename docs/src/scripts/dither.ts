import init, { fill, mix } from '../engine/dither_slint_engine_web.js';

export type Rgba = [number, number, number, number];

export const theme = {
	background: '#1b1a19',
	foreground: '#ebe5d9',
	ink: '#5471f5',
	blue: '#0593e6',
	purple: '#9e60e8',
	green: '#32b364',
	pink: '#e4569e',
	orange: '#ed8725',
	red: '#ea3c3c',
	white: '#ffffff',
	black: '#000000',
} as const;

export const ready = init();

export function rgba(color: string, alpha = 1): Rgba {
	const hex = color in theme ? theme[color as keyof typeof theme] : color;
	const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
	return [r, g, b, Math.round(alpha * 255)];
}

export const css = ([r, g, b, a]: Rgba) => `rgba(${r}, ${g}, ${b}, ${a / 255})`;

export function blend(base: Rgba, other: Rgba, amount: number): Rgba {
	return Array.from(mix(new Uint8Array(base), new Uint8Array(other), amount)) as Rgba;
}

/** A palette entry as the docs write them: a theme name, a hex colour, or `mix:base:other:amount`. */
export function paletteColor(spec: string): Rgba {
	const [kind, base, other, amount] = spec.split(':');
	return kind === 'mix' ? blend(rgba(base), rgba(other), Number(amount)) : rgba(spec);
}

export const variants = { gradient: 0, dotted: 1, hatched: 2, solid: 3 } as const;
export const blooms = { off: 0, low: 1, high: 2 } as const;
export const edges = { top: 0, bottom: 1, left: 2, right: 3 } as const;

export interface Layer {
	variant?: keyof typeof variants;
	edge?: keyof typeof edges;
	length: number;
	from?: number;
	to?: number;
	color: Rgba;
	opacity?: number | string;
	hover?: boolean;
	press?: boolean;
	frame?: boolean;
}

export function layer(box: HTMLElement, step: number, options: Layer) {
	const { variant = 'gradient', edge = 'bottom', length, from = 1, to = 0, color, opacity = 1, hover, press, frame = true } = options;
	const columns = Math.ceil(box.clientWidth / step);
	const rows = Math.ceil(box.clientHeight / step);
	const bytes = fill(columns, rows, variants[variant], edges[edge], length, from, to, new Uint8Array(color), frame);
	const canvas = document.createElement('canvas');
	canvas.width = columns;
	canvas.height = rows;
	canvas.getContext('2d')!.putImageData(new ImageData(new Uint8ClampedArray(bytes), columns, rows), 0, 0);
	canvas.className = hover ? 'layer hover' : press ? 'layer press' : 'layer';
	canvas.style.width = `${columns * step}px`;
	canvas.style.height = `${rows * step}px`;
	canvas.style.setProperty('--opacity', String(opacity));
	box.append(canvas);
}

export function sampleScene(width: number, height: number) {
	const image = new ImageData(width, height);
	for (let y = 0; y < height; y++) {
		for (let x = 0; x < width; x++) {
			const dx = (x - 100) / 48;
			const dy = (y - 65) / 48;
			const inside = dx * dx + dy * dy < 1;
			const z = Math.sqrt(Math.max(0, 1 - dx * dx - dy * dy));
			const lit = Math.max(0, -dx * 0.5 - dy * 0.55 + z * 0.65);
			const floor = y > 98 ? 0.3 + 0.25 * ((y - 98) / 32) : 0;
			const shadow = Math.hypot((x - 112) / 62, (y - 108) / 9) < 1 ? 0.55 : 1;
			const back = floor ? floor * shadow : 0.25 + 0.45 * (1 - y / 98);
			const value = (inside ? 0.12 + 0.88 * lit : back) * 255;
			image.data.set([value, value * 0.92, value * 0.8, 255], (y * width + x) * 4);
		}
	}
	return image;
}
