'use client';

import { useTransition } from 'react';
import { Button } from './ui/button';
import { Wand2 } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { pickAndSelectRandomIdeaForSessionAction } from '@/app/actions';

interface SessionIdeaPickerProps {
  sessionId: string;
  buttonText: string;
  className?: string;
  variant?: 'default' | 'outline' | 'secondary' | 'ghost' | 'link' | 'destructive' | null;
}

export function SessionIdeaPicker({ sessionId, buttonText, className, variant = 'default' }: SessionIdeaPickerProps) {
  const [isPending, startTransition] = useTransition();
  const { toast } = useToast();

  const handlePickIdea = () => {
    startTransition(async () => {
      try {
        const result = await pickAndSelectRandomIdeaForSessionAction({ sessionId });
        if (result?.ideaId) {
          toast({
            title: 'New Idea Selected!',
            description: 'A new idea has been added to the session for workshopping.',
          });
        } else {
          toast({
            title: 'No More Ideas',
            description: 'There are no more submitted ideas to pick from.',
            variant: 'destructive',
          });
        }
      } catch (error) {
        console.error(error);
        toast({
          title: 'An Error Occurred',
          description: 'Could not select a new idea.',
          variant: 'destructive',
        });
      }
    });
  };

  return (
    <Button onClick={handlePickIdea} disabled={isPending} className={className} variant={variant}>
      <Wand2 className={`mr-2 h-4 w-4 ${isPending ? 'animate-spin' : ''}`} />
      {isPending ? 'Picking...' : buttonText}
    </Button>
  );
}
