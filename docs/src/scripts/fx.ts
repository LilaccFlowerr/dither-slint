import { hover, idle, press } from '../engine/dither_slint_engine_web.js';
import type { Rgba } from './dither';

export const hovers = ['sweep', 'spotlight', 'scan', 'trace', 'rise', 'noise', 'none'] as const;
export const presses = ['ripple', 'burst', 'flash', 'pulse', 'glitch', 'none'] as const;
export const idles = ['none', 'shimmer', 'breathe', 'twinkle'] as const;
export const matrices = ['bayer2', 'bayer4', 'bayer8', 'blue-noise'] as const;

export const motion = {
	hover: 'sweep' as (typeof hovers)[number],
	press: 'ripple' as (typeof presses)[number],
	idle: 'none' as (typeof idles)[number],
	matrix: 'bayer4' as (typeof matrices)[number],
	speed: 1,
	intensity: 1,
};

const hoverTime = { sweep: 620, scan: 560, trace: 900, noise: 480 } as const;
const pressTime = { burst: 640, flash: 420, pulse: 720, glitch: 380, ripple: 560 } as const;

export interface FxOptions {
	color: Rgba;
	strength?: number;
	cell?: number;
	effect?: Partial<typeof motion>;
}

function canvasOn(host: HTMLElement) {
	const canvas = document.createElement('canvas');
	canvas.className = 'layer fx';
	canvas.style.opacity = '1';
	canvas.style.display = 'none';
	host.append(canvas);
	return canvas;
}

function draw(canvas: HTMLCanvasElement, bytes: Uint8Array, columns: number, rows: number, step: number) {
	canvas.width = columns;
	canvas.height = rows;
	canvas.style.width = `${columns * step}px`;
	canvas.style.height = `${rows * step}px`;
	canvas.getContext('2d')!.putImageData(new ImageData(new Uint8ClampedArray(bytes), columns, rows), 0, 0);
	canvas.style.display = '';
}

export function attachFx(host: HTMLElement, trigger: HTMLElement, options: FxOptions) {
	const { color, strength = 0.7, cell = 2 } = options;
	const m = () => ({ ...motion, ...options.effect });
	const idleCanvas = canvasOn(host);
	const hoverCanvas = canvasOn(host);
	const pressCanvas = canvasOn(host);
	const color4 = new Uint8Array(color);

	let hovering = false;
	let target = [0, 0];
	let spot = [0, 0];
	let level = 0;
	let enterStart: number | null = null;
	let pressStart: number | null = null;
	let pressAt = [0, 0];
	let seed = 0;
	let shimmerStart: number | null = null;
	let lastShimmer = 0;
	let frame = 0;
	let last = performance.now();

	const pace = () => Math.max(0.1, m().speed);
	const amount = () => Math.min(1, Math.max(0, strength * m().intensity));
	const matrix = () => matrices.indexOf(m().matrix);
	const size = () => [Math.max(1, Math.ceil(host.clientWidth / cell)), Math.max(1, Math.ceil(host.clientHeight / cell))];
	const held = () => m().hover === 'spotlight' || m().hover === 'rise';
	const cellsAt = (event: PointerEvent) => {
		const box = trigger.getBoundingClientRect();
		return [(event.clientX - box.left) / cell, (event.clientY - box.top) / cell];
	};

	function startHover() {
		const name = m().hover;
		if (!held() && name !== 'none' && enterStart === null && name in hoverTime) {
			const started = (enterStart = performance.now());
			window.setTimeout(() => {
				if (enterStart === started) {
					enterStart = null;
					hoverCanvas.style.display = 'none';
				}
			}, hoverTime[name as keyof typeof hoverTime] / pace() + 100);
		}
		schedule();
	}

	const clear = () => {
		for (const canvas of [idleCanvas, hoverCanvas, pressCanvas]) canvas.style.display = 'none';
	};

	function tick(now: number) {
		frame = 0;
		try {
			frameAt(now);
		} catch (error) {
			clear();
			throw error;
		}
	}

	function frameAt(now: number) {
		const dt = Math.max(0, now - last);
		last = now;
		const [columns, rows] = size();
		let busy = false;

		level += ((hovering && held() ? 1 : 0) - level) * Math.min(1, (dt * pace()) / 200);
		spot[0] += (target[0] - spot[0]) * Math.min(1, dt / 70);
		spot[1] += (target[1] - spot[1]) * Math.min(1, dt / 70);

		const hoverName = m().hover;
		let entering = false;
		if (enterStart !== null && hoverName in hoverTime) {
			const total = hoverTime[hoverName as keyof typeof hoverTime];
			entering = (now - enterStart) * pace() < total;
			if (!entering) enterStart = null;
		}
		if (entering || level > 0.01 || (hovering && held())) {
			const total = hoverTime[hoverName as keyof typeof hoverTime] ?? 1;
			const t = held() || enterStart === null ? 0 : ((now - enterStart) * pace()) / total;
			draw(hoverCanvas, hover(hovers.indexOf(hoverName), columns, rows, t, spot[0], spot[1], level, amount(), color4, matrix()), columns, rows, cell);
			busy = true;
		} else {
			hoverCanvas.style.display = 'none';
		}

		const pressName = m().press;
		if (pressStart !== null && pressName in pressTime) {
			const total = pressTime[pressName as keyof typeof pressTime];
			const t = ((now - pressStart) * pace()) / total;
			if (t < 1) {
				draw(pressCanvas, press(presses.indexOf(pressName), columns, rows, t, pressAt[0], pressAt[1], seed, amount(), color4, matrix()), columns, rows, cell);
				busy = true;
			} else {
				pressStart = null;
				pressCanvas.style.display = 'none';
			}
		} else {
			pressCanvas.style.display = 'none';
		}

		const idleName = m().idle;
		if (idleName === 'shimmer' && now - lastShimmer > 3400 / pace()) {
			shimmerStart = now;
			lastShimmer = now;
		}
		let t = -1;
		if (idleName === 'shimmer' && shimmerStart !== null) {
			t = ((now - shimmerStart) * pace()) / 1100;
			if (t > 1) {
				shimmerStart = null;
				t = -1;
			}
		} else if (idleName === 'breathe') {
			t = ((now * pace()) % 2600) / 2600;
		} else if (idleName === 'twinkle') {
			t = Math.floor((now * pace()) / 100);
		}
		if (t >= 0) {
			draw(idleCanvas, idle(idles.indexOf(idleName), columns, rows, t, amount(), color4, cell, matrix()), columns, rows, cell);
		} else {
			idleCanvas.style.display = 'none';
		}
		busy ||= idleName !== 'none';

		if (busy) schedule();
		else clear();
	}

	let timer = 0;

	function run(now?: number) {
		cancelAnimationFrame(frame);
		clearTimeout(timer);
		tick(typeof now === 'number' ? now : performance.now());
	}

	function schedule() {
		if (frame) return;
		last = performance.now();
		frame = requestAnimationFrame(run);
		timer = window.setTimeout(run, 100);
	}

	trigger.addEventListener('pointerenter', (event) => {
		target = spot = cellsAt(event);
		hovering = true;
		startHover();
	});
	trigger.addEventListener('pointermove', (event) => {
		target = cellsAt(event);
		schedule();
	});
	trigger.addEventListener('pointerleave', () => {
		hovering = false;
		schedule();
		window.setTimeout(() => {
			if (!hovering) {
				level = 0;
				if (enterStart === null) hoverCanvas.style.display = 'none';
			}
		}, 1500);
	});
	trigger.addEventListener('pointerdown', (event) => {
		if (m().press === 'none') return;
		pressAt = cellsAt(event);
		seed += 1;
		const started = (pressStart = performance.now());
		const total = pressTime[m().press as keyof typeof pressTime];
		window.setTimeout(() => {
			if (pressStart === started) {
				pressStart = null;
				pressCanvas.style.display = 'none';
			}
		}, total / pace() + 100);
		schedule();
	});
	schedule();
}
