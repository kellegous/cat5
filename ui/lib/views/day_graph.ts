import { Model, Storm } from '../model';
import { El, svg } from '../dom';
import { Day } from '../day';
import { range } from '../range';
import Iter from '../iter';
import { Box } from '../box';

const MONTHS = [
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

class Month {
	constructor(public readonly index: number) {
	}

	beginsOn(): Day {
		return new Day(this.index, 1);
	}

	endsOn(): Day {
		return new Day(this.index + 1, 1);
	}

	name(): string {
		return MONTHS[this.index];
	}
}

function getMonths(): Iter<Month> {
	return Iter.of(range.toExclusive(12)).map(i => new Month(i));
}

function renderTo(
	el: El,
	days: { day: Day, storms: Storm[] }[],
	rect: Box,
) {
	const graphRect = Box.fromTRBL(rect.y + 30, rect.width, rect.height, rect.x + 30),
		dx = graphRect.width / days.length,
		dy = graphRect.height / Iter.of(days).map(({ storms }) => storms.length).max(0);

	{
		const view = Box.fromTRBL(rect.y, graphRect.r, rect.height, graphRect.l);
		for (const month of getMonths()) {
			const index = month.index,
				a = month.beginsOn(),
				b = month.endsOn();
			if ((index & 1) === 0) {
				svg.of('rect')
					.withAttrs({
						x: view.l + dx * a.index,
						y: view.t,
						width: dx * (b.index - a.index),
						height: view.height,
						fill: '#f6f6f6',
					}).appendTo(el);
			}

			svg.of('text')
				.withText(month.name().toLowerCase())
				.withAttrs({
					x: view.l + dx * a.index + 5,
					y: view.t + 5,
					'font-family': 'Lato',
					'font-size': '14px',
					'dominant-baseline': 'hanging',
				}).appendTo(el);
		}
	}

	for (const [i, day] of days.entries()) {
		const x = dx * i,
			y = dy * day.storms.length;
		svg.of('rect')
			.withAttrs({
				x: graphRect.l + x + 1,
				y: graphRect.t + graphRect.height - y,
				width: dx - 2,
				height: y,
				fill: '#09f',
			}).appendTo(el);
	}
}

function groupStormsByDay(
	storms: Storm[]
): { day: Day, storms: Storm[] }[] {
	const days = Iter.of(range.toExclusive(366)).map(i => ({
		day: Day.fromIndex(i),
		storms: [],
	})).collect();

	for (const storm of storms) {
		const { track } = storm,
			n = track.length;
		if (n == 0) {
			continue;
		}
		const a = Day.fromDate(track[0].time).index,
			b = Day.fromDate(track[n - 1].time).index;
		for (let i = a; i <= b; i++) {
			days[i].storms.push(storm);
		}
	}

	return days;
}

export function DayGraph(
	parent: HTMLElement,
	model: Model,
	height: number,
) {
	const width = parent.getBoundingClientRect().width,
		doc = svg.of('svg')
			.withAttrs({
				width: width,
				height: height,
			}).appendTo(parent);

	model.stormsDidLoad.tap(
		storms => renderTo(
			doc,
			groupStormsByDay(storms),
			Box.fromXYWH(0, 0, width, height))
	);
}