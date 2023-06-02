import './header.scss';

import { html, El } from '../dom';

export function Header(
	parent: HTMLElement,
	text: string,
): El {
	return html.of('h1')
		.withClass('header')
		.withText(text)
		.appendTo(parent);
}