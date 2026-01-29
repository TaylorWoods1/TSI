/**
 * @fileoverview Defines the user dashboard page, which provides a summary of
 * the user's submitted ideas and key statistics.
 */
'use client';
import { ArrowUpRight, Lightbulb, PlusCircle } from 'lucide-react';
import Link from 'next/link';

import { PageHeader } from '@/components/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useAuth } from '@/lib/auth';
import { mockIdeas } from '@/lib/data';
import { getIdeaStatusVariant } from '@/lib/utils';

/**
 * The main component for the user dashboard.
 * It displays a welcome message, statistics on idea submissions, and a list of the user's ideas.
 */
export default function Dashboard() {
  const { user } = useAuth();
  // In a real application, this data would be fetched from an API.
  const myIdeas = mockIdeas.filter((idea) => idea.userId === user?.userId);

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title={`Welcome, ${user?.firstName}!`}
        description="Here's a quick overview of your creative contributions."
      >
        <Link href="/ideas/submit" passHref>
          <Button>
            <PlusCircle className="mr-2 h-4 w-4" />
            Submit New Idea
          </Button>
        </Link>
      </PageHeader>

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Total Ideas Submitted
            </CardTitle>
            <Lightbulb className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{myIdeas.length}</div>
            <p className="text-xs text-muted-foreground">
              Thank you for your contributions
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>My Ideas</CardTitle>
          <CardDescription>
            A list of all the ideas you have submitted.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {myIdeas.length > 0 ? (
            <ul className="divide-y divide-border">
              {myIdeas.map((idea) => (
                <li
                  key={idea.ideaId}
                  className="flex items-center justify-between py-3"
                >
                  <div>
                    <p className="font-semibold">{idea.title}</p>
                    <p className="line-clamp-1 text-sm text-muted-foreground">
                      {idea.description}
                    </p>
                  </div>
                  <div className="flex items-center gap-4">
                    <Badge
                      variant={getIdeaStatusVariant(idea.status)}
                      className="capitalize"
                    >
                      {idea.status === 'selectedForSession'
                        ? 'Selected'
                        : idea.status}
                    </Badge>
                    <Button variant="outline" size="sm">
                      View <ArrowUpRight className="ml-2 h-4 w-4" />
                    </Button>
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="py-12 text-center text-muted-foreground">
              <Lightbulb className="mx-auto h-12 w-12" />
              <h3 className="mt-4 text-lg font-semibold">No ideas yet?</h3>
              <p className="mt-1 text-sm">
                It&apos;s time to share your brilliant thoughts!
              </p>
              <Link href="/ideas/submit" passHref>
                <Button className="mt-4">Submit your first idea</Button>
              </Link>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
