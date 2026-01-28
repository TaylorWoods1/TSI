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
import { CalendarIcon } from 'lucide-react';
import { format } from 'date-fns';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Calendar } from '@/components/ui/calendar';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { cn } from '@/lib/utils';

interface EditSessionDialogProps {
  session: IdeationSession | null;
  isNew?: boolean;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedSession: IdeationSession) => void;
}

// A completely rebuilt component to address the UI lock-up bug.
export function EditSessionDialog({ session, isNew, isOpen, onClose, onSave }: EditSessionDialogProps) {
  // Internal state for the form fields to avoid conflicts with parent state
  const [formData, setFormData] = useState<Partial<IdeationSession>>({});
  const { toast } = useToast();

  // Effect to populate the form only when the dialog opens with a valid session
  useEffect(() => {
    if (isOpen && session) {
      setFormData({
        ...session,
        sessionDate: session.sessionDate ? new Date(session.sessionDate) : new Date(),
      });
    }
  }, [isOpen, session]);

  const handleFieldChange = (field: keyof IdeationSession, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleSave = () => {
    if (!formData.name?.trim() || !formData.description?.trim() || !formData.sessionDate) {
      toast({
        title: "Validation Error",
        description: "Please fill out all fields, including the date.",
        variant: "destructive",
      });
      return;
    }
    
    // Pass a properly formatted session object back to the parent
    onSave({
      ...session,
      ...formData,
      sessionId: session!.sessionId,
      sessionDate: new Date(formData.sessionDate).toISOString(),
    } as IdeationSession);
    
    onClose();
  };

  // This handler ensures that any action that should close the dialog
  // (clicking the 'x', pressing Esc, clicking the overlay) correctly
  // signals the parent component.
  const handleOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isNew ? 'Create Session' : 'Edit Session'}</DialogTitle>
          <DialogDescription>
            {isNew
              ? "Provide details for the new session."
              : "Make changes to your session details here. Click save when you're done."}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Session Name</Label>
            <Input 
              id="name" 
              value={formData.name || ''} 
              onChange={(e) => handleFieldChange('name', e.target.value)} 
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={formData.description || ''}
              onChange={(e) => handleFieldChange('description', e.target.value)}
              className="min-h-[100px]"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="date">Session Date</Label>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant={"outline"}
                    className={cn(
                      "w-full justify-start text-left font-normal",
                      !formData.sessionDate && "text-muted-foreground"
                    )}
                  >
                    <CalendarIcon className="mr-2 h-4 w-4" />
                    {formData.sessionDate ? format(new Date(formData.sessionDate), "PPP") : <span>Pick a date</span>}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-auto p-0">
                  <Calendar
                    mode="single"
                    selected={formData.sessionDate ? new Date(formData.sessionDate) : undefined}
                    onSelect={(date) => handleFieldChange('sessionDate', date)}
                    initialFocus
                  />
                </PopoverContent>
              </Popover>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="status">Status</Label>
              <Select 
                value={formData.status || 'planned'} 
                onValueChange={(value: IdeationSession['status']) => handleFieldChange('status', value)}
              >
                  <SelectTrigger id="status">
                      <SelectValue placeholder="Select a status" />
                  </SelectTrigger>
                  <SelectContent>
                      <SelectItem value="planned">Planned</SelectItem>
                      <SelectItem value="active">Active</SelectItem>
                      <SelectItem value="completed">Completed</SelectItem>
                  </SelectContent>
              </Select>
            </div>
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
