/**
 * Helpers for floating layers (tooltips, popovers, selects, modals, toasts).
 * Floating content is portalled to <body> so it is never clipped by an
 * `overflow: hidden` card or a scrolling page container.
 */

/** Svelte action: move the node to document.body while mounted. */
export function portal(node: HTMLElement) {
	document.body.appendChild(node);
	return {
		destroy() {
			node.remove();
		}
	};
}

export type Placement = 'top' | 'bottom' | 'bottom-start' | 'bottom-end' | 'right';

const VIEWPORT_MARGIN = 8;

/**
 * Compute a fixed position for `floating` next to `anchor`, flipping vertically
 * when there is not enough room and clamping to the viewport.
 */
export function computePosition(
	anchor: DOMRect,
	floating: { width: number; height: number },
	placement: Placement,
	offset = 6
) {
	const vw = window.innerWidth;
	const vh = window.innerHeight;
	let top = 0;
	let left = 0;

	if (placement === 'right') {
		left = anchor.right + offset;
		top = anchor.top + anchor.height / 2 - floating.height / 2;
	} else {
		const wantsTop = placement === 'top';
		const roomBelow = vh - anchor.bottom - offset;
		const roomAbove = anchor.top - offset;
		const placeTop = wantsTop
			? roomAbove >= floating.height || roomAbove > roomBelow
			: roomBelow < floating.height && roomAbove > roomBelow;

		top = placeTop ? anchor.top - offset - floating.height : anchor.bottom + offset;

		if (placement === 'bottom-start') {
			left = anchor.left;
		} else if (placement === 'bottom-end') {
			left = anchor.right - floating.width;
		} else {
			left = anchor.left + anchor.width / 2 - floating.width / 2;
		}
	}

	left = Math.min(Math.max(VIEWPORT_MARGIN, left), vw - floating.width - VIEWPORT_MARGIN);
	top = Math.min(Math.max(VIEWPORT_MARGIN, top), vh - floating.height - VIEWPORT_MARGIN);
	return { top: Math.round(top), left: Math.round(left) };
}

let idCounter = 0;
export function uid(prefix = 'ui') {
	idCounter += 1;
	return `${prefix}-${idCounter}`;
}

const FOCUSABLE =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function focusableWithin(root: HTMLElement) {
	return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
		(el) => !el.hasAttribute('disabled') && el.offsetParent !== null
	);
}
