/**
 * @fileoverview The root page of the application, which handles initial routing
 * based on the user's authentication status.
 */
'use client';

import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

/**
 * The main entry page component. It checks if a user is logged in and redirects them
 * to the appropriate page (dashboard or login). It displays a loading spinner
 * while the authentication status is being determined.
 */
export default function Home() {
  const { user, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading) {
      if (user) {
        router.replace('/dashboard');
      } else {
        router.replace('/login');
      }
    }
  }, [user, loading, router]);

  // Display a loading indicator until the redirect happens.
  return (
    <div className="flex h-screen w-full items-center justify-center bg-background">
      <div className="flex flex-col items-center gap-4">
        <div className="h-12 w-12 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
        <p className="text-muted-foreground">Loading Platform...</p>
      </div>
    </div>
  );
}
