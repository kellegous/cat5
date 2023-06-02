export class Signal<T> {
	private listeners: { (arg: T): void }[] = [];

	tap(l: (arg: T) => void): () => void {
		this.listeners = this.listeners.slice(0);
		this.listeners.push(l);
		return () => this.untap(l);
	}

	private untap(l: (arg: T) => void) {
		const ix = this.listeners.indexOf(l);
		if (ix === -1) {
			return;
		}
		this.listeners = this.listeners.slice(0);
		this.listeners.splice(ix, 1);
	}

	raise(arg: T) {
		this.listeners.forEach(l => l.call(this, arg));
	}
}