/**
 * @fileoverview Defines the main layout for authenticated users, providing the
 * sidebar, header, and content area. It also protects routes from unauthenticated access.
 */
'use client';

import React, { useEffect } from 'react';
import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { AppSidebar } from '@/components/app-sidebar';
import { AppHeader } from '@/components/app-header';

/**
 * A layout component that wraps all authenticated pages. It enforces authentication,
 * redirecting to the login page if the user is not logged in. While loading, it

 * displays a spinner.
 *
 * @param {object} props - The component props.
 * @param {React.ReactNode} props.children - The child page component to be rendered.
 * @returns The JSX element for the authenticated layout.
 */
export default function AuthenticatedLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { user, loading } = useAuth();
  const router = useRouter();

  // Effect to handle redirection based on authentication state.
  useEffect(() => {
    if (!loading && !user) {
      router.replace('/login');
    }
  }, [user, loading, router]);

  // Show a loading spinner while authentication status is being determined.
  if (loading || !user) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-background">
        <div className="h-12 w-12 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
      </div>
    );
  }

  // Render the main application layout for authenticated users.
  return (
    <div className="flex min-h-screen w-full bg-muted/40">
      <AppSidebar />
      <div className="flex flex-1 flex-col sm:pl-14">
        <AppHeader />
        <main className="flex-1 p-4 sm:p-6">{children}</main>
      </div>
    </div>
  );
}
