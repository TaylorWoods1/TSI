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
import type { Idea } from '@/lib/types';
import { useState, useEffect } from 'react';
import { useToast } from '@/hooks/use-toast';

interface EditIdeaDialogProps {
  idea: Idea | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedIdea: Idea) => void;
}

export function EditIdeaDialog({ idea, isOpen, onClose, onSave }: EditIdeaDialogProps) {
  const [formData, setFormData] = useState<Partial<Idea>>({});
  const { toast } = useToast();

  useEffect(() => {
    if (isOpen && idea) {
      setFormData(idea);
    }
  }, [isOpen, idea]);

  const handleFieldChange = (field: keyof Idea, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleSave = () => {
    if (!formData.title?.trim() || !formData.description?.trim()) {
      toast({
        title: "Validation Error",
        description: "Please fill out all fields.",
        variant: "destructive",
      });
      return;
    }
    
    onSave({
      ...idea,
      ...formData,
    } as Idea);
    
    onClose();
  };

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Edit Idea</DialogTitle>
          <DialogDescription>
            Make changes to the idea details here. Click save when you're done.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="title">Idea Title</Label>
            <Input 
              id="title" 
              value={formData.title || ''} 
              onChange={(e) => handleFieldChange('title', e.target.value)} 
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={formData.description || ''}
              onChange={(e) => handleFieldChange('description', e.target.value)}
              className="min-h-[150px]"
            />
          </div>
        </div>
        <DialogFooter>
           <Button variant="outline" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSave}>Save Changes</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
