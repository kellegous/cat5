
const FACS = [1, 2, 5];

function expand(s: number, min: number, max: number): number[] {
	const r = [],
		b = Math.floor(min / s) * s;
	for (let i = b + s; i < max; i += s) {
		r.push(i);
	}
	return r;
}

export class Scale {
	private constructor(
		public readonly min: number,
		public readonly max: number,
		public readonly divs: number[],
		public readonly step: number,
	) {
	}

	static for(min: number, max: number, lim: number = 5.5): Scale {
		if (isNaN(min) || isNaN(max)) {
			throw new Error('inputs must be finite');
		}

		if (min === 0 && max === 0) {
			return Scale.for(0, 1, lim);
		}

		const dy = max - min;
		let mag = Math.pow(10, Math.floor(Math.log10(dy) - 1));
		while (true) {
			for (const f of FACS) {
				const s = f * mag;
				if ((dy / s) <= lim) {
					return new this(
						Math.floor(min / s) * s,
						Math.ceil(max / s) * s,
						expand(s, min, max),
						s,
					);
				}
			}
			mag *= 10;
		}
	}
}