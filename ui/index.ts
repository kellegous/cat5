import './index.scss';

import { DOY, MOY } from './lib/time';
import { Model } from './lib/model';

import { CanvasDayGraph } from './lib/views/canvas_day_graph';
import { DayGraph } from './lib/views/day_graph';
import { Header } from './lib/views/header';

const HURRICANE_SEASON: [DOY, DOY] = [
	DOY.fromMD(MOY.June, 1),
	DOY.fromMD(MOY.November, 30),
];

// namespace app {
// 	namespace raw {
// 		export interface Storm {
// 			id: string;
// 			name: string | null;
// 			track: TrackEntry[];
// 		}

// 		export interface TrackEntry {
// 			time: number;
// 			status: string;
// 			location: [number, number];
// 			max_wind: number;
// 			min_pressure: number;
// 			wind_radius_34kt: number;
// 			wind_radius_50kt: number;
// 			wind_radius_64kt: number;
// 		}
// 	}

// 	namespace daygraph {
// 		class Box {
// 			constructor(
// 				public readonly x: number,
// 				public readonly y: number,
// 				public readonly width: number,
// 				public readonly height: number,
// 			) {
// 			}

// 			get l(): number {
// 				return this.x;
// 			}

// 			get r(): number {
// 				const { x, width } = this;
// 				return x + width;
// 			}

// 			get t(): number {
// 				return this.y;
// 			}

// 			get b(): number {
// 				const { y, height } = this;
// 				return y + height;
// 			}

// 			static fromXYWH(
// 				x: number,
// 				y: number,
// 				width: number,
// 				height: number,
// 			): Box {
// 				return new this(x, y, width, height);
// 			}

// 			static fromTRBL(
// 				t: number,
// 				r: number,
// 				b: number,
// 				l: number,
// 			): Box {
// 				return new this(
// 					l,
// 					t,
// 					r - l,
// 					b - t,
// 				);
// 			}
// 		}

// 		type Month = [Day, Day];

// 		function getMonths(): Month[] {
// 			const months = [];
// 			for (let i = 0; i < 12; i++) {
// 				months.push([
// 					new Day(i, 1),
// 					new Day(i + 1, 1),
// 				]);
// 			}
// 			return months;
// 		}

// 		export function renderTo(
// 			parent: HTMLElement,
// 			days: { day: Day, storms: Storm[] }[],
// 			height: number,
// 		): El {
// 			console.log(
// 				getMonths().map(([a, b]) => [new Date(a.time), new Date(b.time)])
// 			);

// 			const width = parent.getBoundingClientRect().width,
// 				doc = svg.of('svg')
// 					.withAttrs({
// 						width: width,
// 						height: height,
// 					}),
// 				dx = width / days.length,
// 				dy = height / Iter.of(days).map(({ storms }) => storms.length).max(0),
// 				view = Box.fromTRBL(10, width - 10, height - 20, 10);

// 			svg.of('rect')
// 				.withAttrs({
// 					x: 0,
// 					y: 0,
// 					width: width,
// 					height: height,
// 					fill: '#f6f6f6',
// 				})
// 				.appendTo(doc);

// 			svg.of('rect')
// 				.withAttrs({
// 					x: view.x,
// 					y: view.y,
// 					width: view.width,
// 					height: view.height,
// 					fill: '#ccc',
// 				})
// 				.appendTo(doc);

// 			for (const [i, day] of days.entries()) {
// 				const x = dx * i,
// 					y = dy * day.storms.length;
// 				svg.of('rect')
// 					.withAttrs({
// 						x: x + 1,
// 						y: height - y,
// 						width: dx - 2,
// 						height: y,
// 						fill: '#09f',
// 					})
// 					.appendTo(doc);
// 			}

// 			return doc.appendTo(parent);
// 		}
// 	}

// 	interface Storm {
// 		id: string;
// 		name: string | null;
// 		year: number;
// 		track: TrackEntry[];
// 	}

// 	interface TrackEntry {
// 		time: Date;
// 		status: string;
// 		location: [number, number];
// 		max_wind: number;
// 		min_pressure: number;
// 		wind_radius_34kt: number;
// 		wind_radius_50kt: number;
// 		wind_radius_64kt: number;
// 	}

// 	function toTrackEntry(entry: raw.TrackEntry): TrackEntry {
// 		const { time, status, location, max_wind, min_pressure, wind_radius_34kt, wind_radius_50kt, wind_radius_64kt } = entry;
// 		return {
// 			time: new Date(time),
// 			status,
// 			location,
// 			max_wind,
// 			min_pressure,
// 			wind_radius_34kt,
// 			wind_radius_50kt,
// 			wind_radius_64kt,
// 		};
// 	}

// 	function toStorm(storm: raw.Storm): Storm {
// 		const { id, name, track } = storm,
// 			year = parseInt(id.substring(4));
// 		return {
// 			id,
// 			name,
// 			year,
// 			track: track.map(toTrackEntry),
// 		};
// 	}

// 	function groupStormsByDay(
// 		storms: Iter<Storm>
// 	): { day: Day, storms: Storm[] }[] {
// 		const days = Iter.of(range.toExclusive(366)).map(i => ({
// 			day: Day.fromIndex(i),
// 			storms: [],
// 		})).collect();

// 		for (const storm of storms) {
// 			const { track } = storm,
// 				n = track.length;
// 			if (n == 0) {
// 				continue;
// 			}
// 			const a = Day.fromDate(track[0].time).index,
// 				b = Day.fromDate(track[n - 1].time).index;
// 			for (let i = a; i <= b; i++) {
// 				days[i].storms.push(storm);
// 			}
// 		}
// 		return days;
// 	}

// 	function formatDay(day: Day): string {
// 		const m = (day.month + 1) + '',
// 			d = day.date + '';
// 		return `${m.padStart(2, '0')}/${d.padStart(2, '0')}`;
// 	}

// 	async function main() {
// 		const storms = await fetch('/data/storms.json')
// 			.then(res => res.json())
// 			.then((storms: raw.Storm[]) => Iter.of(storms).map(toStorm));

// 		const el = document.querySelector('#app') as HTMLElement,
// 			days = groupStormsByDay(storms),
// 			max = days.reduce(
// 				(ix, day, i) => day.storms.length > days[ix].storms.length ? i : ix,
// 				0
// 			);

// 		daygraph.renderTo(el, days, 300);

// 		for (const [i, day] of days.entries()) {
// 			const row = html.of('div')
// 				.withClass('day')
// 				.append(
// 					html.of('div')
// 						.withClass('date')
// 						.withText(formatDay(day.day))
// 				)
// 				.append(
// 					html.of('div')
// 						.withClass('count')
// 						.withText(day.storms.length.toLocaleString())
// 				)
// 				.appendTo(el);

// 			if (i === max) {
// 				row.withClass('max');
// 			}
// 		}
// 	}

// main();

const model = new Model().load(),
	root = document.querySelector('#app') as HTMLElement;

Header(root, 'Storms by Day of Year');
DayGraph(root, model, {
	height: 300,
});

Header(root, 'Storms by Day of Year (Hurricane Season)');
DayGraph(root, model, {
	height: 300,
	startDay: DOY.fromMD(MOY.June, 1),
	endDay: DOY.fromMD(MOY.November, 30),
});

Header(root, "Storms by Day of Year");
CanvasDayGraph(root, model, {
	season: HURRICANE_SEASON,
	height: 300,
});