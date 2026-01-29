/**
 * @fileoverview Defines the layout for the admin section, protecting it from
 * unauthorized access and providing the standard app navigation.
 */
'use client';

import React, { useEffect } from 'react';
import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { AppSidebar } from '@/components/app-sidebar';
import { AppHeader } from '@/components/app-header';
import { AlertCircle } from 'lucide-react';

/**
 * A layout component that wraps all admin pages. It enforces that the user
 * is not only authenticated but also has the 'administrator' role.
 *
 * @param {object} props - The component props.
 * @param {React.ReactNode} props.children - The admin page component to be rendered.
 * @returns The JSX element for the admin layout.
 */
export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading) {
      if (!user) {
        // Not logged in, redirect to login page.
        router.replace('/login');
      } else if (user.role !== 'administrator') {
        // Logged in but not an admin, redirect to user dashboard.
        router.replace('/dashboard');
      }
    }
  }, [user, loading, router]);

  // Show a loading spinner while authentication status is being checked.
  if (loading || !user) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-background">
        <div className="h-12 w-12 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
      </div>
    );
  }

  // Show an access denied message while redirecting non-admin users.
  if (user.role !== 'administrator') {
    return (
        <div className="flex h-screen w-full items-center justify-center bg-background">
            <div className="flex flex-col items-center gap-4 p-4 text-center">
                <AlertCircle className="h-12 w-12 text-destructive" />
                <h2 className="text-2xl font-bold">Access Denied</h2>
                <p className="text-muted-foreground">You do not have permission to view this page.</p>
                 <p className="text-sm text-muted-foreground">Redirecting...</p>
            </div>
      </div>
    );
  }
  
  // If user is an admin, render the standard application layout.
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
