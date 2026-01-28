import { PageHeader } from '@/components/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { mockSessions } from '@/lib/data';
import { cn } from '@/lib/utils';
import { ArrowUpRight, Calendar } from 'lucide-react';
import Link from 'next/link';

export default function SessionsPage() {
  const getStatusClass = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800 border-green-200 dark:bg-green-900/50 dark:text-green-300 dark:border-green-800';
      case 'planned':
        return 'bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/50 dark:text-blue-300 dark:border-blue-800';
      case 'completed':
        return 'bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-700';
      default:
        return '';
    }
  };

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Ideation Sessions"
        description="Explore upcoming, active, and past innovation sessions."
      />
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {mockSessions.map((session) => (
          <Card key={session.sessionId}>
            <CardHeader>
              <div className="flex justify-between items-start">
                  <CardTitle className="line-clamp-2">{session.name}</CardTitle>
                  <Badge className={cn('capitalize', getStatusClass(session.status))}>
                      {session.status}
                  </Badge>
              </div>
              <CardDescription className="flex items-center gap-2 pt-2 text-sm text-muted-foreground">
                <Calendar className="h-4 w-4" />
                {new Date(session.sessionDate).toLocaleDateString()}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="line-clamp-3 text-sm text-muted-foreground">
                {session.description}
              </p>
            </CardContent>
            <CardFooter>
              <Link href={`/sessions/${session.sessionId}`} passHref className="w-full">
                <Button variant="outline" className="w-full">
                  {session.status === 'active' ? 'Join Session' : 'View Details'}
                  <ArrowUpRight className="ml-2 h-4 w-4" />
                </Button>
              </Link>
            </CardFooter>
          </Card>
        ))}
      </div>
    </div>
  );
}
