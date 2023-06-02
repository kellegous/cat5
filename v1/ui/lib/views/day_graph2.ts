import { DOY, MOY } from '../time';
import { svg, El } from '../dom';
import { Model, Storm } from '../model';
import { Queue } from '../transition';
import { range } from '../range';
import Iter from '../iter';

const DEFAULT_HEIGHT = 300;

interface Data {
	days: { day: DOY, storms: Storm[] }[];
	months: { month: MOY, a: number, b: number }[];
	season: [DOY, DOY];
}

function toData(
	storms: Storm[],
	season: [DOY, DOY],
): Data {
	const days = Iter.of(range.toExclusive(366))
		.map(i => ({
			day: DOY.fromIndex(i),
			storms: [],
		})).collect();

	for (const storm of storms) {
		const { track } = storm,
			n = track.length;
		if (n === 0) {
			continue;
		}
		const a = DOY.fromDate(track[0].time).index,
			b = DOY.fromDate(track[n - 1].time).index;
		for (let i = a; i <= b; i++) {
			days[i].storms.push(storm);
		}
	}

	const months = Iter.of(getMonthsIn(MOY.January, MOY.December))
		.map(m => ({
			month: m,
			a: m.firstDay().index,
			b: m.lastDay().index,
		})).collect();

	return { days, months, season };
}

function* getMonthsIn(start: MOY, end: MOY): Generator<MOY> {
	yield start;
	for (let m = start.next();
		m.index <= end.index && m !== MOY.January;
		m = m.next()
	) {
		yield m;
	}
}

namespace view {
	export interface ForMonth {
		region: El | null,
		label: El,
		month: MOY,
	}
}

class View {
	private data: Data | null = null;

	private zoom = 0.0;

	private queue = new Queue;

	constructor(
		private doc: El,
		model: Model,
		season: [DOY, DOY],
	) {
		model.stormsDidLoad.tap(
			storms => this.dataDidLoad(toData(storms, season))
		);
	}

	private dataDidLoad(data: Data) {
	}
}

export interface DayGraphOptions {
	height?: number;
	season: [DOY, DOY];
}

export function DayGraph(
	parent: HTMLElement,
	model: Model,
	opts: DayGraphOptions,
): El {
	const width = parent.getBoundingClientRect().width,
		height = opts.height || DEFAULT_HEIGHT,
		doc = svg.of('svg')
			.withAttrs({
				width: width,
				height: height,
				viewBox: `0 0 ${width} ${height}`,
			})
			.appendTo(parent);
	new View(doc, model, opts.season);
	return doc;
}