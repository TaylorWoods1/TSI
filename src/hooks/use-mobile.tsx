/**
 * @fileoverview Defines a custom hook, useIsMobile, to detect if the application
 * is being viewed on a mobile-sized screen.
 */
import * as React from "react"

const MOBILE_BREAKPOINT = 768; // Standard breakpoint for tablets

/**
 * A custom React hook that returns `true` if the window's width is below a
 * defined mobile breakpoint. It listens for window resize events to provide
 * a reactive boolean value.
 *
 * This hook is useful for conditionally rendering components or applying different
 * styles for mobile vs. desktop views.
 *
 * @returns {boolean} `true` if the screen is mobile-sized, `false` otherwise.
 * Returns `undefined` during server-side rendering before the `useEffect` runs.
 */
export function useIsMobile() {
  const [isMobile, setIsMobile] = React.useState<boolean | undefined>(undefined)

  React.useEffect(() => {
    const mql = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`)
    
    const onChange = () => {
      setIsMobile(window.innerWidth < MOBILE_BREAKPOINT)
    }

    // Set the initial value
    onChange();

    // Add listener for changes
    mql.addEventListener("change", onChange)

    return () => {
      // Clean up the listener when the component unmounts
      mql.removeEventListener("change", onChange)
    }
  }, [])

  return isMobile;
}
