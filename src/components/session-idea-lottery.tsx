'use client';

import { useState, useTransition } from 'react';
import { Button } from './ui/button';
import { Card, CardContent, CardTitle, CardDescription, CardHeader, CardFooter } from './ui/card';
import { Wand2, Ticket, CheckCircle, Lightbulb } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useToast } from '@/hooks/use-toast';
import { selectIdeasForSessionAction } from '@/app/actions';
import type { Idea } from '@/lib/types';

export function SessionIdeaLottery({ sessionId, availableIdeas, isAdmin }: { sessionId: string; availableIdeas: Idea[], isAdmin: boolean }) {
    const [selectedIdeas, setSelectedIdeas] = useState<string[]>([]);
    const [isPicking, setIsPicking] = useState(false);
    const [spinning, setSpinning] = useState(false);
    const [finalSelection, setFinalSelection] = useState<string[]>([]);
    const [isPending, startTransition] = useTransition();

    const { toast } = useToast();

    const isLoading = isPicking || isPending;

    const handleToggleSelection = (ideaId: string) => {
        if (isLoading || !isAdmin) return;
        setSelectedIdeas(prev =>
            prev.includes(ideaId) ? prev.filter(id => id !== ideaId) : [...prev, ideaId]
        );
    };

    const runLottery = (ideas: Idea[], count: number) => {
        setIsPicking(true);
        setSpinning(true);
        setFinalSelection([]);
        setSelectedIdeas([]);

        const allIdeaIds = ideas.map(i => i.ideaId);
        let currentIndex = 0;
        const interval = setInterval(() => {
            if(allIdeaIds.length > 0) {
                setFinalSelection([allIdeaIds[currentIndex % allIdeaIds.length]]);
                currentIndex++;
            }
        }, 100);

        setTimeout(() => {
            clearInterval(interval);
            const shuffled = [...ideas].sort(() => 0.5 - Math.random());
            const picked = shuffled.slice(0, Math.min(count, shuffled.length)).map(i => i.ideaId);

            setFinalSelection(picked);
            setSpinning(false);
            
            setTimeout(() => {
                startTransition(async () => {
                    await selectIdeasForSessionAction({ sessionId, ideaIds: picked });
                    toast({
                        title: `${picked.length} Ideas Selected!`,
                        description: `The winning ideas have been chosen for the session.`
                    });
                });
            }, 1500);
        }, 3000);
    };
    
    const handlePickRandomly = () => {
        if (availableIdeas.length < 1) {
            toast({ title: "Not enough ideas", description: "There are no submitted ideas to pick from.", variant: "destructive"});
            return;
        }
        const count = Math.min(availableIdeas.length, Math.floor(Math.random() * 3) + 1);
        runLottery(availableIdeas, count);
    };
    
    const handleConfirmSelection = () => {
        if (selectedIdeas.length === 0) {
            toast({ title: "No ideas selected", description: "Please select at least one idea to confirm.", variant: "destructive"});
            return;
        }
        startTransition(async () => {
            await selectIdeasForSessionAction({ sessionId, ideaIds: selectedIdeas });
            toast({
                title: `${selectedIdeas.length} Ideas Confirmed!`,
                description: `You have manually selected ideas for the session.`
            });
        });
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex items-center gap-3"><Lightbulb className="h-6 w-6 text-primary"/>Idea Selection</CardTitle>
                <CardDescription>
                    {isAdmin ? "Select ideas for this session manually, or start a lottery to pick them randomly." : "The idea selection is in progress. Get ready for the big reveal!"}
                </CardDescription>
            </CardHeader>
            <CardContent>
                {availableIdeas.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-1">
                        {availableIdeas.map(idea => (
                            <Card
                                key={idea.ideaId}
                                onClick={() => handleToggleSelection(idea.ideaId)}
                                className={cn(
                                    "transition-all duration-300 relative overflow-hidden",
                                    isAdmin && "cursor-pointer",
                                    !isAdmin && "cursor-default",
                                    selectedIdeas.includes(idea.ideaId) && !isPicking && "ring-2 ring-primary ring-offset-2",
                                    spinning && !finalSelection.includes(idea.ideaId) && "opacity-50 scale-95",
                                    finalSelection.includes(idea.ideaId) && "bg-green-100 dark:bg-green-900/50 ring-2 ring-green-500 animate-pulse",
                                    isLoading && "cursor-not-allowed"
                                )}
                            >
                                <CardContent className="p-4">
                                    <CardTitle className="text-base font-bold line-clamp-2">{idea.title}</CardTitle>
                                    <CardDescription className="text-xs line-clamp-3 mt-2">{idea.description}</CardDescription>
                                    {isAdmin && selectedIdeas.includes(idea.ideaId) && !isPicking && (
                                        <div className="absolute top-2 right-2 h-5 w-5 bg-primary rounded-full flex items-center justify-center">
                                            <CheckCircle className="h-5 w-5 text-primary-foreground" />
                                        </div>
                                    )}
                                </CardContent>
                            </Card>
                        ))}
                    </div>
                ) : (
                     <div className="flex flex-col items-center justify-center h-full text-muted-foreground py-12">
                        <Lightbulb className="mx-auto h-12 w-12" />
                        <h3 className="mt-4 text-lg font-semibold">No Submitted Ideas</h3>
                        <p className="text-sm mt-1">There are no ideas with a 'submitted' status to select from.</p>
                    </div>
                )}
            </CardContent>
            {isAdmin && (
                <CardFooter className="border-t pt-6 justify-between">
                     <p className="text-sm text-muted-foreground">
                        {availableIdeas.length > 0 ? "Ready to decide?" : "Waiting for ideas..."}
                    </p>
                    <div className="flex gap-2">
                         <Button variant="ghost" onClick={handlePickRandomly} disabled={isLoading || availableIdeas.length === 0}>
                            <Ticket className={`mr-2 h-4 w-4 ${isPicking ? 'animate-spin' : ''}`} />
                            {isPicking ? 'Picking...' : 'Start Lottery'}
                        </Button>
                        <Button onClick={handleConfirmSelection} disabled={isLoading || selectedIdeas.length === 0}>
                            <CheckCircle className="mr-2 h-4 w-4" />
                            Confirm {selectedIdeas.length > 0 ? `(${selectedIdeas.length})` : ''} Selection
                        </Button>
                    </div>
                </CardFooter>
            )}
        </Card>
    );
}
