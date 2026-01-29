/**
 * @fileoverview Provides authentication context for the application, managing user state
 * and providing functions for login and logout.
 */
'use client';

import React, { createContext, useContext, useState, ReactNode, useEffect } from 'react';
import type { User } from './types';
import { mockAdmin } from './data';

/**
 * The shape of the authentication context, including the current user, loading state,
 * and functions to log in and out.
 */
interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (role: 'employee' | 'administrator') => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

/**
 * A provider component that wraps the application and makes authentication state
 * available to any child components. It simulates a persistent session using localStorage.
 *
 * @param {object} props - The component props.
 * @param {ReactNode} props.children - The child components that need access to the auth context.
 */
export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate checking for a logged-in user in localStorage on initial load.
    try {
      const storedUserRole = localStorage.getItem('userRole');
      if (storedUserRole && (storedUserRole === 'employee' || storedUserRole === 'administrator')) {
        setUser({ ...mockAdmin, role: storedUserRole });
      }
    } catch (error) {
      console.error('Could not access localStorage:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const login = (role: 'employee' | 'administrator') => {
    // In a real app, this would involve a call to an auth provider and storing a token.
    const loggedInUser: User = { ...mockAdmin, role };
    setUser(loggedInUser);
    try {
      localStorage.setItem('userRole', role);
    } catch (error) {
      console.error('Could not access localStorage:', error);
    }
  };

  const logout = () => {
    setUser(null);
    try {
      localStorage.removeItem('userRole');
    } catch (error) {
      console.error('Could not access localStorage:', error);
    }
  };

  const value = { user, loading, login, logout };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/**
 * A custom hook for accessing the authentication context.
 * It provides an easy way to get the current user, loading state, and auth functions.
 * Throws an error if used outside of an `AuthProvider`.
 *
 * @returns The authentication context.
 */
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
