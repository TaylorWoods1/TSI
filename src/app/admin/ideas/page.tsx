import { PageHeader } from '@/components/page-header';
import { mockIdeas, mockUsers } from '@/lib/data';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { MoreHorizontal } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';

const statusVariant = (status: string): 'default' | 'secondary' | 'outline' | 'destructive' => {
  switch (status) {
    case 'selectedForSession':
      return 'default';
    case 'submitted':
      return 'secondary';
    case 'archived':
      return 'outline';
    default:
      return 'outline';
  }
};

export default function AdminIdeasPage() {
  const ideasWithUsers = mockIdeas.map((idea) => ({
    ...idea,
    user: idea.isAnonymous ? null : mockUsers.find((u) => u.userId === idea.userId),
  }));

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Idea Management"
        description="View, filter, and manage all ideas submitted to the platform."
      />
      <Card>
        <CardHeader>
          <CardTitle>All Ideas</CardTitle>
          <CardDescription>A comprehensive list of every idea.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Title</TableHead>
                <TableHead>Author</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Date Submitted</TableHead>
                <TableHead>
                  <span className="sr-only">Actions</span>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {ideasWithUsers.map((idea) => (
                <TableRow key={idea.ideaId}>
                  <TableCell className="font-medium">{idea.title}</TableCell>
                  <TableCell>
                    {idea.isAnonymous ? (
                      <span className="text-muted-foreground italic">Anonymous</span>
                    ) : (
                      `${idea.user?.firstName} ${idea.user?.lastName}`
                    )}
                  </TableCell>
                  <TableCell>
                    <Badge variant={statusVariant(idea.status)} className="capitalize">
                      {idea.status === 'selectedForSession' ? 'Selected' : idea.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{new Date(idea.submissionDate).toLocaleDateString()}</TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button aria-haspopup="true" size="icon" variant="ghost">
                          <MoreHorizontal className="h-4 w-4" />
                          <span className="sr-only">Toggle menu</span>
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuLabel>Actions</DropdownMenuLabel>
                        <DropdownMenuItem>View Details</DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>Set as Submitted</DropdownMenuItem>
                        <DropdownMenuItem>Archive</DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
