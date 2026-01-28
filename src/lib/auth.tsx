'use client';

import React, { createContext, useContext, useState, ReactNode, useEffect } from 'react';
import type { User } from './types';
import { mockAdmin } from './data';

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (role: 'employee' | 'administrator') => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate checking for a logged-in user
    const storedUserRole = localStorage.getItem('userRole');
    if (storedUserRole) {
      setUser({ ...mockAdmin, role: storedUserRole as 'employee' | 'administrator' });
    }
    setLoading(false);
  }, []);

  const login = (role: 'employee' | 'administrator') => {
    // In a real app, you'd get user details from your auth provider
    const loggedInUser: User = { ...mockAdmin, role };
    setUser(loggedInUser);
    localStorage.setItem('userRole', role);
  };

  const logout = () => {
    setUser(null);
    localStorage.removeItem('userRole');
  };

  const value = { user, loading, login, logout };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
