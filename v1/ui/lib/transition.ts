export type EasingFn = (p: number) => number;

export namespace easing {
	export function linear(p: number): number {
		return p;
	}

	export function inOutQuart(p: number): number {
		return (p < 0.5) ? 8 * p * p * p * p : 1 - Math.pow(-2 * p + 2, 4) / 2;
	}

	export function inOutCubic(p: number): number {
		return (p < 0.5) ? 4 * p * p * p : 1 - Math.pow(-2 * p + 2, 3) / 2;
	}

	export function inExpo(p: number): number {
		return (p < 0.5) ? 4 * p * p * p : 1 - Math.pow(-2 * p + 2, 3) / 2;
	}

	export function inOutSine(p: number): number {
		return -(Math.cos(Math.PI * p) - 1) / 2;
	}

	export function inOutCirc(p: number): number {
		return p < 0.5
			? (1 - Math.sqrt(1 - Math.pow(2 * p, 2))) / 2
			: (Math.sqrt(1 - Math.pow(-2 * p + 2, 2)) + 1) / 2;
	}

	export function inOutBack(x: number): number {
		const c1 = 1.70158;
		const c2 = c1 * 1.525;
		return x < 0.5
			? (Math.pow(2 * x, 2) * ((c2 + 1) * 2 * x - c2)) / 2
			: (Math.pow(2 * x - 2, 2) * ((c2 + 1) * (x * 2 - 2) + c2) + 2) / 2;
	}

	export function inOutElastic(x: number): number {
		const c5 = (2 * Math.PI) / 4.5;
		return x === 0
			? 0
			: x === 1
				? 1
				: x < 0.5
					? -(Math.pow(2, 20 * x - 10) * Math.sin((20 * x - 11.125) * c5)) / 2
					: (Math.pow(2, -20 * x + 10) * Math.sin((20 * x - 11.125) * c5)) / 2 + 1;
	}
}

export class Transition<T> {
	private constructor(
		public readonly fn: (p: number, state: T) => void,
		public readonly duration: number,
		public readonly init: () => T,
	) {
	}

	static of<T>(
		fn: (p: number, state: T) => void,
		duration: number,
		ease: EasingFn = easing.linear,
		init: () => T = () => null,
	) {
		return new this((
			p: number, state: T) => fn(ease(p), state),
			duration,
			init,
		);
	}
}

export class Queue {
	private active: Transition<any> | null = null;
	private pending: Transition<any>[] = [];

	add<T>(t: Transition<T>) {
		this.pending.push(t);
		if (!this.active) {
			this.start();
		}
	}

	private start() {
		const { pending } = this;
		if (pending.length === 0) {
			this.active = null;
			return;
		}

		const current = pending.shift(),
			{ duration, fn, init } = current,
			ta = Date.now(),
			state = init(),
			tick = () => {
				const p = Math.min(1.0, (Date.now() - ta) / duration);
				fn(Math.min(1.0, (Date.now() - ta) / duration), state);
				if (p < 1) {
					requestAnimationFrame(tick);
				} else {
					fn(1, state);
					this.start();
				}
			};
		fn(0, state);
		this.active = current;
		requestAnimationFrame(tick);
	}
}
