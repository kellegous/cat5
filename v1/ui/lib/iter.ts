function* withIndex<T>(
	iter: Iterable<T>,
): Generator<[number, T]> {
	let i = 0;
	for (const v of iter) {
		yield [i, v];
		i++;
	}
}

function* map<T, V, U>(
	iter: Iterable<T>,
	fn: (T, number?, U?) => V,
	state?: U,
): Iterable<V> {
	for (const [i, v] of withIndex(iter)) {
		yield fn(v, i, state);
	}
}

function* take<T>(
	iter: Iterable<T>,
	n: number,
): Iterable<T> {
	let count = 0;
	for (const v of iter) {
		if (count >= n) {
			break;
		}
		yield v;
		count++;
	}
}

export default class Iter<T> implements Iterable<T> {
	constructor(
		private iter: Iterable<T>) {
	}

	[Symbol.iterator]() {
		return this.iter[Symbol.iterator]();
	}

	map<V, U>(
		fn: (T, number?, U?) => V,
		state?: U,
	): Iter<V> {
		return new Iter(map(this.iter, fn, state));
	}

	reduce<V>(
		fn: (V, T, number?) => V,
		value: V,
	): V {
		for (const [i, v] of withIndex(this.iter)) {
			value = fn(value, v, i);
		}
		return value;
	}

	forEach(fn: (T, number?) => void) {
		for (const [i, v] of withIndex(this.iter)) {
			fn(v, i);
		}
	}

	take(n: number): Iter<T> {
		return new Iter(take(this.iter, n));
	}

	collect(): T[] {
		return Array.from(this.iter);
	}

	max(d: T | null): T | null {
		return this.reduce(
			(max, v) => v > max ? v : max,
			d
		);
	}

	enumerate(): Iter<[number, T]> {
		return new Iter(withIndex(this.iter));
	}

	static of<T>(iter: Iterable<T>): Iter<T> {
		return new Iter(iter);
	}
}