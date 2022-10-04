const DAY_IN_MILLIS = 86400 * 1000;

export class Day {
	private constructor(
		public readonly month: number,
		public readonly date: number,
	) {
	}

	get index(): number {
		const t1 = Date.UTC(2000, this.month, this.date),
			t0 = Date.UTC(2000, 1, 1);
		return ((t1 - t0) / DAY_IN_MILLIS) | 0;
	}

	static fromDate(date: Date): Day {
		return new this(
			date.getMonth(),
			date.getDate()
		);
	}

	static fromIndex(index: number): Day {
		const t = new Date(Date.UTC(2000, 1, 1) + index * DAY_IN_MILLIS);
		return new this(
			t.getUTCMonth(),
			t.getUTCDate()
		);
	}
}