/**
 * @fileoverview Defines the login page for user authentication.
 */
'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { useAuth } from '@/lib/auth';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useToast } from '@/hooks/use-toast';

/**
 * A page component that provides a login form. It includes a role switcher
 * to simulate logging in as either an 'employee' or an 'administrator'.
 */
export default function LoginPage() {
  const [role, setRole] = useState<'employee' | 'administrator'>('employee');
  const { login } = useAuth();
  const router = useRouter();
  const { toast } = useToast();

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();
    toast({
      title: 'Login Successful',
      description: `You are now logged in as an ${role}.`,
    });
    login(role);
    // Redirect based on the selected role for this simulation.
    router.push(role === 'administrator' ? '/sessions' : '/dashboard');
  };

  return (
    <Card className="w-full max-w-sm">
      <form onSubmit={handleLogin}>
        <CardHeader>
          <CardTitle className="text-2xl">Login</CardTitle>
          <CardDescription>Enter your credentials to access your account.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="email">Email</Label>
            <Input id="email" type="email" placeholder="m@example.com" defaultValue="admin@tsi.com" required />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="password">Password</Label>
            <Input id="password" type="password" defaultValue="password" required />
          </div>
          <div className="grid gap-2">
            <Label>Role (for simulation)</Label>
            <RadioGroup defaultValue="employee" value={role} onValueChange={(value) => setRole(value as 'employee' | 'administrator')}>
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="employee" id="r1" />
                <Label htmlFor="r1">Employee</Label>
              </div>
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="administrator" id="r2" />
                <Label htmlFor="r2">Administrator</Label>
              </div>
            </RadioGroup>
          </div>
        </CardContent>
        <CardFooter className="flex-col gap-4">
          <Button className="w-full" type="submit">
            Sign In
          </Button>
          <div className="text-sm text-center">
            <Link href="/forgot-password" passHref>
              <span className="underline cursor-pointer">Forgot your password?</span>
            </Link>
          </div>
          <div className="mt-4 text-center text-sm">
            Don&apos;t have an account?{' '}
            <Link href="/register" passHref>
              <span className="underline cursor-pointer">Sign up</span>
            </Link>
          </div>
        </CardFooter>
      </form>
    </Card>
  );
}
