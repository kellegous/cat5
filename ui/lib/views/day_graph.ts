import { Model, Storm } from '../model';
import { El, svg } from '../dom';
import { Day } from '../day';
import { range } from '../range';
import Iter from '../iter';
import { Box } from '../box';

function renderTo(
	el: El,
	days: { day: Day, storms: Storm[] }[],
	width: number,
	height: number,
) {
	const dx = width / days.length,
		dy = height / Iter.of(days).map(({ storms }) => storms.length).max(0),
		view = Box.fromTRBL(10, width - 10, height - 20, 10);

	svg.of('rect')
		.withAttrs({
			x: 0,
			y: 0,
			width: width,
			height: height,
			fill: '#f6f6f6',
		}).appendTo(el);

	svg.of('rect')
		.withAttrs({
			x: view.x,
			y: view.y,
			width: view.width,
			height: view.height,
			fill: '#ccc',
		}).appendTo(el);

	for (const [i, day] of days.entries()) {
		const x = dx * i,
			y = dy * day.storms.length;
		svg.of('rect')
			.withAttrs({
				x: x + 1,
				y: height - y,
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
		storms => renderTo(doc, groupStormsByDay(storms), width, height)
	);
}