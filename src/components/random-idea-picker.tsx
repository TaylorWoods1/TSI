'use client';

import { useState, useEffect } from 'react';
import { Button } from './ui/button';
import { Wand2 } from 'lucide-react';
import { pickRandomIdeaAction } from '@/app/actions';
import { useToast } from '@/hooks/use-toast';
import { mockIdeas } from '@/lib/data';

export function RandomIdeaPicker({ sessionId }: { sessionId: string }) {
  const [isPicking, setIsPicking] = useState(false);
  const [pickedIdea, setPickedIdea] = useState<string | null>(null);
  const { toast } = useToast();

  const handlePickIdea = async () => {
    setIsPicking(true);
    setPickedIdea(null);
    try {
      const result = await pickRandomIdeaAction({ sessionId });
      if (result?.ideaId) {
        // In a real app, revalidation would handle this. We simulate it.
        setTimeout(() => {
          setPickedIdea(result.ideaId);
          setIsPicking(false);
           toast({
            title: "Idea Selected!",
            description: `"${mockIdeas.find(i=>i.ideaId === result.ideaId)?.title}" has been chosen.`,
          });
        }, 2000); // Wait for animation
      } else {
        toast({
          title: 'No Ideas Available',
          description: 'There are no submitted ideas to pick from.',
          variant: 'destructive',
        });
        setIsPicking(false);
      }
    } catch (error) {
      toast({
        title: 'An Error Occurred',
        description: 'Could not pick a random idea.',
        variant: 'destructive',
      });
      setIsPicking(false);
    }
  };

  if (pickedIdea) {
    return <span className="text-sm font-semibold">{pickedIdea}</span>
  }

  return (
    <Button onClick={handlePickIdea} disabled={isPicking} size="sm" variant="outline">
      <Wand2 className={`mr-2 h-4 w-4 ${isPicking ? 'animate-spin' : ''}`} />
      {isPicking ? 'Picking...' : 'Pick Idea'}
    </Button>
  );
}
