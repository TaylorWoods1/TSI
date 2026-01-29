/**
 * @fileoverview Defines the "Forgot Password" page for handling password reset requests.
 */
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import Link from 'next/link';

/**
 * A page component that displays a form for users to request a password reset link.
 * Note: The form submission is currently a placeholder.
 */
export default function ForgotPasswordPage() {
  return (
    <Card className="w-full max-w-sm">
      <CardHeader>
        <CardTitle className="text-2xl">Forgot Password</CardTitle>
        <CardDescription>Enter your email to receive a password reset link.</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-4">
        <div className="grid gap-2">
          <Label htmlFor="email">Email</Label>
          <Input id="email" type="email" placeholder="m@example.com" required />
        </div>
        <Button type="submit" className="w-full">
          Send Reset Link
        </Button>
        <div className="mt-4 text-center text-sm">
          Remembered your password?{' '}
          <Link href="/login">
            <span className="underline cursor-pointer">Sign in</span>
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
