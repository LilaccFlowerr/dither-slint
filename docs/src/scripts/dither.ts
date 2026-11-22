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
