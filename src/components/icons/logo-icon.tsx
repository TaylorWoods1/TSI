/**
 * @fileoverview Defines the LogoIcon component, a reusable SVG for the application logo.
 */

import type { SVGProps } from 'react';

/**
 * Renders the application's logo as an SVG element.
 * @param props - Standard SVG properties.
 * @returns A JSX element representing the logo.
 */
export function LogoIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <path d="M15.5 3.84a1 1 0 0 1 1.28-1.28l4.33 4.33a1 1 0 0 1-1.28 1.28L15.5 3.84z" />
      <path d="M12.5 6.84a1 1 0 0 0-1.28-1.28l-4.33 4.33a1 1 0 0 0 1.28 1.28L12.5 6.84z" />
      <path d="m14 17 3-3" />
      <path d="M10 17v-3.5" />
      <path d="M7 11v1.5" />
      <path d="m7 17 3-3" />
      <path d="M17 14h-1.5" />
      <circle cx="12" cy="12" r="10" />
    </svg>
  );
}
