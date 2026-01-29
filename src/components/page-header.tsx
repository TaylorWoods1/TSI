/**
 * @fileoverview Defines a standard PageHeader component for consistent page titles and descriptions.
 */
import React from 'react';

/**
 * Props for the PageHeader component.
 */
type PageHeaderProps = {
  /** The main title of the page. */
  title: string;
  /** A short description or subtitle displayed below the title. */
  description: string;
  /** Optional child elements, typically buttons or actions, displayed to the right of the title. */
  children?: React.ReactNode;
};

/**
 * A reusable component for displaying a consistent header at the top of a page.
 * It includes a title, a description, and an area for action buttons.
 * @param props - The properties for the component.
 * @returns A JSX element representing the page header.
 */
export function PageHeader({ title, description, children }: PageHeaderProps) {
  return (
    <div className="mb-6 flex items-start justify-between">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
        <p className="mt-1 text-muted-foreground">{description}</p>
      </div>
      <div className="flex items-center gap-2">{children}</div>
    </div>
  );
}
