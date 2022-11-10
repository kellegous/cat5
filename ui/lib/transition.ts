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
