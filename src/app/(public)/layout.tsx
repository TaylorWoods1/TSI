/**
 * @fileoverview Defines the public layout for pages accessible to unauthenticated users,
 * such as login, register, and forgot password.
 */
import { LogoIcon } from '@/components/icons/logo-icon';

/**
 * A layout component for public-facing pages. It provides a consistent, centered
 * structure with the application logo.
 *
 * @param {object} props - The component props.
 * @param {React.ReactNode} props.children - The child components to be rendered within the layout.
 * @returns The JSX element for the public layout.
 */
export default function PublicLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen w-full flex-col items-center justify-center bg-background p-4">
      <div className="mb-8 flex items-center gap-3 text-2xl font-bold text-primary">
        <LogoIcon className="h-8 w-8" />
        Tech Sol Innovations
      </div>
      {children}
    </div>
  );
}
