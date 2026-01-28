'use client';

import { useState } from 'react';
import { PageHeader } from '@/components/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { mockSessions as initialMockSessions } from '@/lib/data';
import { cn } from '@/lib/utils';
import {
  ArrowUpRight,
  Calendar,
  MoreHorizontal,
  PlusCircle,
} from 'lucide-react';
import Link from 'next/link';
import { useAuth } from '@/lib/auth';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { EditSessionDialog } from '@/components/edit-session-dialog';
import { DeleteSessionAlert } from '@/components/delete-session-alert';
import type { IdeationSession } from '@/lib/types';
import { useToast } from '@/hooks/use-toast';

export default function SessionsPage() {
  const { user } = useAuth();
  const { toast } = useToast();
  const [sessions, setSessions] = useState(() => JSON.parse(JSON.stringify(initialMockSessions)));

  // State for the edit dialog
  const [editingSession, setEditingSession] = useState<IdeationSession | null>(null);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [isCreatingNew, setIsCreatingNew] = useState(false);

  // State for the delete alert
  const [deletingSession, setDeletingSession] = useState<IdeationSession | null>(null);
  const [isDeleteOpen, setIsDeleteOpen] = useState(false);

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

  const handleSaveSession = (updatedSession: IdeationSession) => {
    const isNew = !sessions.some(s => s.sessionId === updatedSession.sessionId);
    if (isNew) {
        setSessions(prev => [updatedSession, ...prev]);
    } else {
        setSessions(prev => prev.map(s => s.sessionId === updatedSession.sessionId ? updatedSession : s));
    }
    toast({
        title: isNew ? "Session Created" : "Session Updated",
        description: `The "${updatedSession.name}" session has been saved.`,
    });
    setIsEditOpen(false);
  };

  const handleConfirmDelete = () => {
    if (!deletingSession) return;
    const sessionName = sessions.find(s => s.sessionId === deletingSession.sessionId)?.name;
    setSessions(prev => prev.filter(s => s.sessionId !== deletingSession.sessionId));
    toast({
        title: "Session Deleted",
        description: `The "${sessionName}" session has been removed.`,
        variant: "destructive"
    });
    setIsDeleteOpen(false);
  };

  const handleTriggerCreate = () => {
    const newSession: IdeationSession = {
      sessionId: `session-${Date.now()}`,
      name: 'New Session',
      description: 'A new ideation session.',
      sessionDate: new Date().toISOString(),
      selectedIdeaIds: [],
      status: 'planned',
    };
    setIsCreatingNew(true);
    setEditingSession(newSession);
    setIsEditOpen(true);
  };
  
  const handleTriggerEdit = (session: IdeationSession) => {
    setIsCreatingNew(false);
    setEditingSession(session);
    setIsEditOpen(true);
  };
  
  const handleTriggerDelete = (session: IdeationSession) => {
    setDeletingSession(session);
    setIsDeleteOpen(true);
  };

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Ideation Sessions"
        description="Explore upcoming, active, and past innovation sessions."
      >
        {user?.role === 'administrator' && (
          <Button onClick={handleTriggerCreate}>
            <PlusCircle className="mr-2 h-4 w-4" />
            Create Session
          </Button>
        )}
      </PageHeader>
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {sessions.map((session) => (
          <Card key={session.sessionId}>
            <CardHeader>
              <div className="flex items-start justify-between">
                <CardTitle className="line-clamp-2">{session.name}</CardTitle>
                <div className="flex shrink-0 items-center gap-1">
                  <Badge
                    className={cn('capitalize', getStatusClass(session.status))}
                  >
                    {session.status}
                  </Badge>
                  {user?.role === 'administrator' && (
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
                        <DropdownMenuItem onClick={() => handleTriggerEdit(session)}>Edit</DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleTriggerDelete(session)} className="text-destructive focus:bg-destructive/10 focus:text-destructive">
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  )}
                </div>
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
              <Link
                href={`/sessions/${session.sessionId}`}
                passHref
                className="w-full"
              >
                <Button variant="outline" className="w-full">
                  {session.status === 'active'
                    ? 'Join Session'
                    : 'View Details'}
                  <ArrowUpRight className="ml-2 h-4 w-4" />
                </Button>
              </Link>
            </CardFooter>
          </Card>
        ))}
      </div>

       <EditSessionDialog
        session={editingSession}
        isNew={isCreatingNew}
        isOpen={isEditOpen}
        onClose={() => setIsEditOpen(false)}
        onSave={handleSaveSession}
      />
      
      <DeleteSessionAlert
        session={deletingSession}
        isOpen={isDeleteOpen}
        onClose={() => setIsDeleteOpen(false)}
        onConfirm={handleConfirmDelete}
      />
    </div>
  );
}
