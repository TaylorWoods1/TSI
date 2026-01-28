'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { PageHeader } from '@/components/page-header';
import { useToast } from '@/hooks/use-toast';
import { useRouter } from 'next/navigation';

export default function SubmitIdeaPage() {
  const { toast } = useToast();
  const router = useRouter();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    toast({
      title: 'Idea Submitted!',
      description: 'Thank you for your contribution to innovation.',
      variant: 'default',
    });
    router.push('/dashboard');
  };

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Submit an Idea"
        description="Your next great idea could shape our future. Share it with us."
      />

      <Card>
        <form onSubmit={handleSubmit}>
          <CardContent className="pt-6">
            <div className="grid gap-6">
              <div className="grid gap-3">
                <Label htmlFor="title">Idea Title</Label>
                <Input id="title" placeholder="e.g., AI-Powered Workflow Automation" required />
                <p className="text-sm text-muted-foreground">
                  Give your idea a short, descriptive title.
                </p>
              </div>
              <div className="grid gap-3">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  placeholder="Describe the problem, your proposed solution, and its potential benefits..."
                  className="min-h-[150px]"
                  required
                />
                 <p className="text-sm text-muted-foreground">
                  Be as detailed as you can. What problem does this solve? How does it work?
                </p>
              </div>
              <div className="flex items-center space-x-3">
                <Switch id="anonymous-submission" />
                <div className="grid gap-0.5">
                  <Label htmlFor="anonymous-submission">Submit Anonymously</Label>
                  <p className="text-sm text-muted-foreground">
                    If toggled, your name will not be attached to this idea.
                  </p>
                </div>
              </div>
            </div>
          </CardContent>
          <CardHeader>
            <Button type="submit">Submit Idea</Button>
          </CardHeader>
        </form>
      </Card>
    </div>
  );
}
