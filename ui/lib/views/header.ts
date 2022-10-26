import './header.scss';

import { html } from '../dom';

export function Header(
	parent: HTMLElement,
	text: string,
) {
	html.of('h1')
		.withClass('header')
		.withText(text)
		.appendTo(parent);
}