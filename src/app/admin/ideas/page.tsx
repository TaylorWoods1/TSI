/**
 * @fileoverview Defines the admin page for managing all submitted ideas.
 */
'use client';

import { useState } from 'react';
import { MoreHorizontal } from 'lucide-react';

import { EditIdeaDialog } from '@/components/edit-idea-dialog';
import { PageHeader } from '@/components/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useToast } from '@/hooks/use-toast';
import { mockIdeas, mockUsers } from '@/lib/data';
import type { Idea } from '@/lib/types';
import { getIdeaStatusVariant } from '@/lib/utils';

// Augment the Idea type with optional user details for display purposes.
type IdeaWithUser = Idea & {
  user: (typeof mockUsers)[0] | null;
};

/**
 * The main component for the idea management page.
 * It displays a table of all ideas, allowing administrators to view, edit, and update their status.
 */
export default function AdminIdeasPage() {
  const { toast } = useToast();
  // Deep copy mock data to allow for mutation in state. In a real app, this would be managed via API calls.
  const [ideas, setIdeas] = useState<Idea[]>(() =>
    JSON.parse(JSON.stringify(mockIdeas))
  );
  const [ideaToEdit, setIdeaToEdit] = useState<IdeaWithUser | null>(null);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);

  // Join idea data with user data for displaying author names.
  const ideasWithUsers: IdeaWithUser[] = ideas.map((idea: Idea) => ({
    ...idea,
    user: idea.isAnonymous
      ? null
      : mockUsers.find((u) => u.userId === idea.userId) || null,
  }));

  const handleTriggerEdit = (idea: IdeaWithUser) => {
    // Timeout to prevent race condition with dropdown menu closing
    setTimeout(() => {
      setIdeaToEdit(idea);
      setIsEditDialogOpen(true);
    }, 150);
  };

  const handleSaveIdea = (updatedIdea: Idea) => {
    setIdeas((prevIdeas: Idea[]) =>
      prevIdeas.map((idea) =>
        idea.ideaId === updatedIdea.ideaId ? updatedIdea : idea
      )
    );
    toast({
      title: 'Idea Updated',
      description: `The idea "${updatedIdea.title}" has been saved.`,
    });
  };

  const handleUpdateStatus = (ideaId: string, status: Idea['status']) => {
    const ideaTitle = ideas.find((i: Idea) => i.ideaId === ideaId)?.title;
    setIdeas((prevIdeas: Idea[]) =>
      prevIdeas.map((idea) =>
        idea.ideaId === ideaId ? { ...idea, status } : idea
      )
    );
    toast({
      title: 'Status Updated',
      description: `"${ideaTitle}" is now ${
        status === 'selectedForSession' ? 'selected' : status
      }.`,
    });
  };

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Idea Management"
        description="View, filter, and manage all ideas submitted to the platform."
      />
      <Card>
        <CardContent className="pt-6">
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
                      <span className="italic text-muted-foreground">
                        Anonymous
                      </span>
                    ) : (
                      `${idea.user?.firstName} ${idea.user?.lastName}`
                    )}
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={getIdeaStatusVariant(idea.status)}
                      className="capitalize"
                    >
                      {idea.status === 'selectedForSession'
                        ? 'Selected'
                        : idea.status}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    {new Date(idea.submissionDate).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          aria-haspopup="true"
                          size="icon"
                          variant="ghost"
                        >
                          <MoreHorizontal className="h-4 w-4" />
                          <span className="sr-only">Toggle menu</span>
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuLabel>Actions</DropdownMenuLabel>
                        <DropdownMenuItem
                          onClick={() => handleTriggerEdit(idea)}
                        >
                          Edit Idea
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {idea.status !== 'submitted' && (
                          <DropdownMenuItem
                            onClick={() =>
                              handleUpdateStatus(idea.ideaId, 'submitted')
                            }
                          >
                            Set as Submitted
                          </DropdownMenuItem>
                        )}
                        {idea.status === 'archived' ? (
                          <DropdownMenuItem
                            onClick={() =>
                              handleUpdateStatus(idea.ideaId, 'submitted')
                            }
                          >
                            Unarchive
                          </DropdownMenuItem>
                        ) : (
                          <DropdownMenuItem
                            onClick={() =>
                              handleUpdateStatus(idea.ideaId, 'archived')
                            }
                          >
                            Archive
                          </DropdownMenuItem>
                        )}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <EditIdeaDialog
        idea={ideaToEdit}
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        onSave={handleSaveIdea}
      />
    </div>
  );
}
