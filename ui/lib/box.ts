export class Box {
	constructor(
		public readonly x: number,
		public readonly y: number,
		public readonly width: number,
		public readonly height: number,
	) {
	}

	get l(): number {
		return this.x;
	}

	get r(): number {
		const { x, width } = this;
		return x + width;
	}

	get t(): number {
		return this.y;
	}

	get b(): number {
		const { y, height } = this;
		return y + height;
	}

	static fromXYWH(
		x: number,
		y: number,
		width: number,
		height: number,
	): Box {
		return new this(x, y, width, height);
	}

	static fromTRBL(
		t: number,
		r: number,
		b: number,
		l: number,
	): Box {
		return new this(
			l,
			t,
			r - l,
			b - t,
		);
	}
}