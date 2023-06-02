const DAY_IN_MILLIS = 86400 * 1000;

const MOY_NAMES = [
	'Jan',
	'Feb',
	'Mar',
	'Apr',
	'May',
	'Jun',
	'Jul',
	'Aug',
	'Sep',
	'Oct',
	'Nov',
	'Dec',
];

export class DOY {
	private constructor(
		public readonly month: MOY,
		public readonly date: number,
	) {
	}

	get index(): number {
		const t1 = Date.UTC(2000, this.month.index, this.date),
			t0 = Date.UTC(2000, 0, 1);
		return ((t1 - t0) / DAY_IN_MILLIS) | 0;
	}

	static fromDate(date: Date): DOY {
		return new this(
			MOY.fromIndex(date.getMonth()),
			date.getDate()
		);
	}

	static fromIndex(index: number): DOY {
		const t = new Date(Date.UTC(2000, 0, 1) + index * DAY_IN_MILLIS);
		return new this(
			MOY.fromIndex(t.getUTCMonth()),
			t.getUTCDate()
		);
	}

	static fromMD(month: MOY, date: number): DOY {
		const t = new Date(2000, month.index, date);
		return new this(MOY.fromIndex(t.getMonth()), t.getDate());
	}
}

export class MOY {
	static readonly January = new MOY(0);
	static readonly February = new MOY(1);
	static readonly March = new MOY(2);
	static readonly April = new MOY(3);
	static readonly May = new MOY(4);
	static readonly June = new MOY(5);
	static readonly July = new MOY(6);
	static readonly August = new MOY(7);
	static readonly September = new MOY(8);
	static readonly October = new MOY(9);
	static readonly November = new MOY(10);
	static readonly December = new MOY(11);

	private static values = [
		this.January,
		this.February,
		this.March,
		this.April,
		this.May,
		this.June,
		this.July,
		this.August,
		this.September,
		this.October,
		this.November,
		this.December,
	];

	private constructor(
		public readonly index: number
	) {
	}

	next(): MOY {
		return MOY.fromIndex(this.index + 1);
	}

	firstDay(): DOY {
		return DOY.fromMD(this, 1);
	}

	lastDay(): DOY {
		return DOY.fromMD(this.next(), 0);
	}

	name(): string {
		return MOY_NAMES[this.index];
	}

	static fromIndex(index: number): MOY {
		return this.values[((index % 12) + 12) % 12];
	}
}