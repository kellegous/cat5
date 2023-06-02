export namespace range {
	function* rangeIn(
		a: number,
		b: number,
		step: number,
	): Generator<number> {
		if (b > a) {
			for (let i = a; i <= b; i += step) {
				yield i;
			}
		} else if (a > b) {
			for (let i = a; i >= b; i += step) {
				yield i;
			}
		} else {
			yield a;
		}
	}

	function* rangeEx(
		a: number,
		b: number,
		step: number,
	): Generator<number> {
		if (b > a) {
			for (let i = a; i < b; i += step) {
				yield i;
			}
		} else if (a > b) {
			for (let i = a; i > b; i += step) {
				yield i;
			}
		}
	}

	export class Iter implements Iterable<number> {
		constructor(
			private a: number = 0,
			private b: number = Number.MAX_VALUE,
			private step: number = 1,
			private fn: (a: number, b: number, step: number) => Generator<number> = rangeEx,
		) {
		}

		[Symbol.iterator]() {
			const { a, b, step } = this;
			return this.fn(a, b, step);
		}

		toInclusive(b: number): Iter {
			const { a, step } = this;
			return new Iter(a, b, step, rangeIn);
		}

		toExclusive(b: number): Iter {
			const { a, step } = this;
			return new Iter(a, b, step, rangeEx);
		}

		from(a: number): Iter {
			const { b, step, fn } = this;
			return new Iter(a, b, step, fn);
		}
	}

	export function from(a: number): Iter {
		return new Iter(a);
	}

	export function toInclusive(b: number): Iter {
		return new Iter(0, b, 1, rangeIn);
	}

	export function toExclusive(b: number): Iter {
		return new Iter(0, b, 1, rangeEx);
	}
}