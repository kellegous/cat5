type NativeEl = HTMLElement | SVGElement;

function toNative<T extends NativeEl>(el: El<T> | NativeEl): NativeEl {
	return (el instanceof El)
		? el.el
		: el;
}

export class El<T extends NativeEl = NativeEl> {
	constructor(
		public readonly el: T,
	) {
	}

	withAttrs(attrs: { [key: string]: any }): El<T> {
		const { el } = this;
		for (const [key, val] of Object.entries(attrs)) {
			el.setAttribute(key, val);
		}
		return this;
	}

	withCSS(props: { [key: string]: any }): El<T> {
		const { el } = this,
			style = el.style;
		for (const [key, val] of Object.entries(props)) {
			style.setProperty(key, val, '');
		}
		return this;
	}

	withClass(c: string): El<T> {
		this.el.classList.add(c);
		return this;
	}

	withText(t: string): El<T> {
		this.el.textContent = t;
		return this;
	}

	appendTo<V extends NativeEl>(el: El<V> | NativeEl): El<T> {
		toNative(el).appendChild(this.el);
		return this;
	}

	append<V extends NativeEl>(el: El<V> | NativeEl): El<T> {
		this.el.appendChild(toNative(el));
		return this;
	}

	prependTo<V extends NativeEl>(el: El<V> | NativeEl): El<T> {
		el = toNative(el);
		el.insertBefore(this.el, el.firstChild);
		return this;
	}

	prepend<V extends NativeEl>(el: El<V> | NativeEl): El<T> {
		this.el.insertBefore(toNative(el), this.el.firstChild);
		return this;
	}

	on(
		name: string,
		fn: EventListenerOrEventListenerObject,
		capture?: boolean | undefined,
	): El<T> {
		this.el.addEventListener(name, fn, capture);
		return this;
	}

	static from<T extends NativeEl>(el: T): El<T> {
		return new this(el);
	}

	static of<T extends NativeEl>(name: string, ns: string = ''): El<T> {
		const el = (ns === '')
			? document.createElement(name)
			: document.createElementNS(ns, name);
		return new this(el as T);
	}
}

export namespace html {
	export function of<T extends HTMLElement = HTMLElement>(
		name: string
	): El<T> {
		return El.of<T>(name);
	}

	export function from<T extends HTMLElement = HTMLElement>(
		el: T
	): El<T> {
		return El.from(el);
	}
}

export namespace svg {
	export const NS = 'http://www.w3.org/2000/svg';

	export function of<T extends SVGElement = SVGElement>(
		name: string
	): El<T> {
		return El.of<T>(name, NS);
	}

	export function from<T extends SVGElement = SVGElement>(
		el: T,
	): El<T> {
		return El.from(el);
	}
}