import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import type { Idea } from './types';
import type { BadgeProps } from '@/components/ui/badge';

/**
 * A utility function to merge Tailwind CSS classes, handling conflicts and removing duplicates.
 * @param inputs - A list of class values to be merged.
 * @returns A string of merged class names.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Determines the variant for the Badge component based on an idea's status.
 * This centralizes the status-to-style mapping.
 * @param status - The status of the idea ('selectedForSession', 'submitted', 'archived').
 * @returns The corresponding badge variant.
 */
export const getIdeaStatusVariant = (
  status: Idea['status']
): BadgeProps['variant'] => {
  switch (status) {
    case 'selectedForSession':
      return 'default';
    case 'submitted':
      return 'secondary';
    case 'archived':
      return 'outline';
    default:
      return 'outline';
  }
};
