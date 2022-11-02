import { Model, Storm } from '../model';
import { El, svg } from '../dom';
import { range } from '../range';
import Iter from '../iter';
import { Box } from '../box';
import { Scale } from '../scale';
import { DOY, MOY } from '../time';

const DEFAULT_HEIGHT = 300;

function renderTo(
	el: El,
	data: Data,
	rect: Box,
) {
	const days = data.days,
		graphRect = Box.fromTRBL(rect.y + 30, rect.width, rect.height, rect.x),
		scale = Scale.for(0, Iter.of(days).map(({ storms }) => storms.length).max(0)),
		dx = graphRect.width / days.length,
		dy = graphRect.height / scale.max;

	{ // months
		const view = Box.fromTRBL(rect.y, graphRect.r, rect.height, graphRect.l),
			months = data.months;
		for (let i = 0, n = months.length; i < n; i++) {
			const { month, a, b } = months[i];
			if ((i & 1) === 0) {
				svg.of('rect')
					.withAttrs({
						x: view.l + dx * a,
						y: view.t,
						width: dx * (b - a + 1),
						height: view.height,
						fill: '#f6f6f6',
					}).appendTo(el);
			}
			svg.of('text')
				.withText(month.name().toLowerCase())
				.withAttrs({
					x: view.l + dx * a + 5,
					y: view.t + 5,
					'font-family': 'Lato',
					'font-size': '14px',
					'dominant-baseline': 'hanging',
				}).appendTo(el);
		}
	}

	{ // y-axis
		const view = Box.fromTRBL(rect.y, graphRect.r, rect.height, graphRect.l);
		for (const div of scale.divs) {
			const y = view.t + view.height - dy * div;
			svg.of('line')
				.withAttrs({
					x1: view.l,
					y1: y,
					x2: view.r,
					y2: y,
					stroke: '#999',
					'stroke-dasharray': '1, 3',
				}).appendTo(el);

			svg.of('text')
				.withText(`${div.toLocaleString()}`)
				.withAttrs({
					x: view.l + 5,
					y: y - 5,
					'font-family': 'Lato',
					'font-size': '10px',
					fill: '#999',
				}).appendTo(el);
		}
	}

	{ // bar graph
		const padding = 0.5;
		for (const [i, day] of days.entries()) {
			const x = dx * i,
				y = dy * day.storms.length;
			svg.of('rect')
				.withAttrs({
					x: graphRect.l + x + padding,
					y: graphRect.t + graphRect.height - y,
					width: dx - padding * 2,
					height: y,
					fill: '#09f',
				}).appendTo(el);
		}
	}
}

interface Data {
	days: { day: DOY, storms: Storm[] }[];
	months: { month: MOY, a: number, b: number }[];
}

function toData(
	storms: Storm[],
	start: DOY,
	end: DOY,
): Data {
	const si = start.index,
		ei = end.index,
		n = (ei - si) + 1,
		days = Iter.of(range.toExclusive(n))
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
			if (i >= si && i <= ei) {
				days[i - si].storms.push(storm);
			}
		}
	}

	const months = Iter.of(getMonthsIn(start.month, end.month))
		.map((m: MOY) => ({
			month: m,
			a: Math.max(m.firstDay().index, si) - si,
			b: Math.min(m.lastDay().index, ei) - si,
		}))
		.collect();

	return {
		days: days,
		months: months,
	};
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

export interface DayGraphOptions {
	startDay?: DOY;
	endDay?: DOY;
	height?: number;
}

export function DayGraph(
	parent: HTMLElement,
	model: Model,
	opts: DayGraphOptions,
): El {
	const width = parent.getBoundingClientRect().width,
		height = opts.height || DEFAULT_HEIGHT,
		startDay = opts.startDay || DOY.fromMD(MOY.January, 1),
		endDay = opts.endDay || DOY.fromMD(MOY.December, 31),
		doc = svg.of('svg')
			.withClass('day-graph')
			.withAttrs({
				width: width,
				height: height,
			}).appendTo(parent);

	model.stormsDidLoad.tap(
		storms => renderTo(
			doc,
			toData(storms, startDay, endDay),
			Box.fromXYWH(0, 0, width, height))
	);

	return doc;
}