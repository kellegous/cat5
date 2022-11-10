import { DOY, MOY } from '../time';
import { html, El } from '../dom';
import { Model, Storm } from '../model';
import { range } from '../range';
import Iter from '../iter';
import { Box } from '../box';
import { Scale } from '../scale';
import { Queue, Transition, easing } from '../transition';

const DEFAULT_HEIGHT = 300;

function lerp(a: number, b: number, p: number): number {
	return (1 - p) * a + p * b;
}

function renderTo(
	canvas: El,
	data: Data,
	rect: Box,
	zoom: number = 0,
) {
	const ctx = (canvas.el as HTMLCanvasElement).getContext('2d'),
		{ months, days, season } = data,
		graphRect = Box.fromTRBL(rect.y + 30, rect.width, rect.height, rect.x),
		scale = Scale.for(0, Iter.of(days).map(({ storms }) => storms.length).max(0)),
		[sa, sb] = season,
		dxa = graphRect.width / days.length,
		dxb = graphRect.width / (sb.index - sa.index + 1),
		dx = lerp(dxa, dxb, zoom),
		offset = lerp(0, -dx * sa.index, zoom),
		dy = graphRect.height / scale.max;

	ctx.clearRect(rect.x, rect.y, rect.width, rect.height);

	{ // months
		const view = Box.fromTRBL(rect.y, graphRect.r, rect.height, graphRect.l);
		for (let i = 0, n = months.length; i < n; i++) {
			const { month, a, b } = months[i];
			if ((i & 1) === 0) {
				ctx.fillStyle = '#f6f6f6';
				ctx.beginPath();
				ctx.rect(
					offset + view.l + dx * a,
					view.t,
					dx * (b - a + 1),
					view.height,
				);
				ctx.fill();
			}

			ctx.fillStyle = '#333';
			ctx.font = '14px Lato';
			const text = month.name().toLowerCase(),
				exts = ctx.measureText(text);
			ctx.fillText(
				month.name().toLowerCase(),
				offset + view.l + dx * a + 5,
				view.t + exts.fontBoundingBoxAscent + 5,
			);
		}
	}

	{ // season
		const [a, b] = data.season,
			view = graphRect,
			x1 = offset + view.l + a.index * dx,
			x2 = offset + view.l + b.index * dx + dx,
			y = view.t + 20,
			alpha = 1 - easing.inExpo(zoom);
		ctx.save();
		ctx.strokeStyle = `rgb(51, 51, 51, ${alpha})`
		ctx.lineCap = 'round';
		ctx.beginPath();
		ctx.moveTo(x1, y + 10);
		ctx.lineTo(x1, y);
		ctx.lineTo(x2, y);
		ctx.lineTo(x2, y + 10);
		ctx.stroke();
		ctx.restore();
	}

	{ // y-axis
		const view = Box.fromTRBL(rect.y, graphRect.r, rect.height, graphRect.l);
		ctx.save();
		ctx.setLineDash([1, 3]);
		ctx.strokeStyle = '#999';
		ctx.beginPath();
		for (const div of scale.divs) {
			const y = view.t + view.height - dy * div;
			ctx.moveTo(offset + view.l, y);
			ctx.lineTo(view.r, y);
		}
		ctx.stroke();
		ctx.restore();

		ctx.save();
		ctx.fillStyle = '#999';
		ctx.font = '10px Lato';
		for (const div of scale.divs) {
			const text = div.toLocaleString(),
				exts = ctx.measureText(text),
				y = view.t + view.height - dy * div;
			ctx.fillText(
				text,
				view.l + 5,
				y - 5,
			);
		}
		ctx.restore();
	}

	{ // bar graph
		const padding = 0.5;
		for (const [i, day] of days.entries()) {
			const x = dx * i,
				y = dy * day.storms.length;
			ctx.fillStyle = '#09f';
			ctx.beginPath();
			ctx.rect(
				offset + graphRect.l + x + padding,
				graphRect.b - y,
				dx - padding * 2,
				y,
			);
			ctx.fill();
		}
	}
}

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

function getTargetTransform(
	width: number,
	data: Data,
): [number, number] {
	const { days, season } = data,
		[a, b] = season,
		n = b.index - a.index + 1;
	return [-a.index * width / n, data.days.length / n];
}

export interface CanvasDayGraphOptions {
	season: [DOY, DOY];
	height?: number;
}

class View {
	private data: Data | null = null;

	private zoom = 0.0;

	private queue = new Queue;

	constructor(
		private canvas: El,
		model: Model,
		season: [DOY, DOY],
	) {
		model.stormsDidLoad.tap(
			storms => this.dataDidLoad(toData(storms, season))
		);
		canvas.on('click', () => this.canvasWasClicked());
	}

	private canvasWasClicked() {
		if (this.data === null) {
			return;
		}

		this.queue.add(
			Transition.of<number>(
				(p, state) => {
					this.zoom = (state == 0) ? p : 1 - p;
					this.render();
				},
				500,
				easing.inOutQuart,
				() => this.zoom,
			)
		);
	}

	private dataDidLoad(data: Data) {
		this.data = data;
		this.render();
	}

	private render() {
		const { data, canvas, zoom } = this;
		if (data === null) {
			return;
		}

		const { width, height } = canvas.el as HTMLCanvasElement,
			now = Date.now();
		renderTo(
			canvas,
			data,
			Box.fromXYWH(0, 0, width, height),
			zoom,
		);
	}
}

export function CanvasDayGraph(
	parent: HTMLElement,
	model: Model,
	opts: CanvasDayGraphOptions,
): El {
	const width = parent.getBoundingClientRect().width,
		height = opts.height || DEFAULT_HEIGHT,
		canvas = html.of('canvas').withClass('canvas-day-graph')
			.withAttrs({
				width: width,
				height: height,
			})
			.appendTo(parent),
		view = new View(canvas, model, opts.season);
	return canvas;
}