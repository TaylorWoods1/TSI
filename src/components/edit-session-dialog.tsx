'use client';

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import type { IdeationSession } from '@/lib/types';
import { useState, useEffect } from 'react';
import { useToast } from '@/hooks/use-toast';

interface EditSessionDialogProps {
  session: IdeationSession | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedSession: IdeationSession) => void;
}

export function EditSessionDialog({ session, isOpen, onClose, onSave }: EditSessionDialogProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const { toast } = useToast();

  const isNewSession = session && !mockSessions.some(s => s.sessionId === session.sessionId);

  useEffect(() => {
    if (session) {
      setName(session.name);
      setDescription(session.description);
    }
  }, [session]);

  const handleSave = () => {
    if (!session) return;
    
    if (!name.trim() || !description.trim()) {
        toast({
            title: "Validation Error",
            description: "Please fill out all fields.",
            variant: "destructive"
        });
        return;
    }

    onSave({ ...session, name, description });
  };

  if (!session) return null;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isNewSession ? 'Create Session' : 'Edit Session'}</DialogTitle>
          <DialogDescription>
            {isNewSession
              ? "Provide details for the new session."
              : "Make changes to your session details here. Click save when you're done."}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Session Name</Label>
            <Input id="name" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="min-h-[100px]"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave}>Save Changes</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// Need to import mockSessions to check if the session is new, this is a workaround for local state management
import { mockSessions } from '@/lib/data';
