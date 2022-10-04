import Day from './lib/day';

namespace app {
	namespace raw {
		export interface Storm {
			id: string;
			name: string | null;
			track: TrackEntry[];
		}

		export interface TrackEntry {
			time: number;
			status: string;
			location: [number, number];
			max_wind: number;
			min_pressure: number;
			wind_radius_34kt: number;
			wind_radius_50kt: number;
			wind_radius_64kt: number;
		}
	}

	interface Storm {
		id: string;
		name: string | null;
		year: number;
		track: TrackEntry[];
	}

	interface TrackEntry {
		time: Date;
		status: string;
		location: [number, number];
		max_wind: number;
		min_pressure: number;
		wind_radius_34kt: number;
		wind_radius_50kt: number;
		wind_radius_64kt: number;
	}

	function toTrackEntry(entry: raw.TrackEntry): TrackEntry {
		const { time, status, location, max_wind, min_pressure, wind_radius_34kt, wind_radius_50kt, wind_radius_64kt } = entry;
		return {
			time: new Date(time),
			status,
			location,
			max_wind,
			min_pressure,
			wind_radius_34kt,
			wind_radius_50kt,
			wind_radius_64kt,
		};
	}

	function toStorm(storm: raw.Storm): Storm {
		const { id, name, track } = storm,
			year = parseInt(id.substring(4));
		return {
			id,
			name,
			year,
			track: track.map(toTrackEntry),
		};
	}

	async function main() {
		const data = await fetch('/data/storms.json')
			.then(res => res.json())
			.then((storms: raw.Storm[]) => storms.map(toStorm));
		console.log(data);
	}

	main();
}