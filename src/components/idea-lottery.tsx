'use client';

import { useState, useEffect, useTransition } from 'react';
import { Button } from './ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter, DialogDescription } from './ui/dialog';
import { Card, CardContent, CardTitle, CardDescription } from './ui/card';
import { Wand2, Ticket, CheckCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useToast } from '@/hooks/use-toast';
import { selectIdeasForSessionAction } from '@/app/actions';
import type { Idea } from '@/lib/types';

export function IdeaLottery({ sessionId, availableIdeas }: { sessionId: string; availableIdeas: Idea[] }) {
    const [isOpen, setIsOpen] = useState(false);
    const [selectedIdeas, setSelectedIdeas] = useState<string[]>([]);
    const [isPicking, setIsPicking] = useState(false);
    const [spinning, setSpinning] = useState(false);
    const [finalSelection, setFinalSelection] = useState<string[]>([]);
    const [isPending, startTransition] = useTransition();

    const { toast } = useToast();

    const isLoading = isPicking || isPending;

    const handleToggleSelection = (ideaId: string) => {
        if (isLoading) return;
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
                    setIsOpen(false);
                    setIsPicking(false);
                });
            }, 1500);
        }, 3000);
    };
    
    const handlePickRandomly = () => {
        if (availableIdeas.length < 1) {
            toast({ title: "Not enough ideas", description: "There are no submitted ideas to pick from.", variant: "destructive"});
            return;
        }
        // pick between 1 and 3 ideas, but not more than available
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
            setIsOpen(false);
        });
    }
    
    // reset state when opening
    useEffect(() => {
        if (isOpen) {
            setSelectedIdeas([]);
            setIsPicking(false);
            setSpinning(false);
            setFinalSelection([]);
        }
    }, [isOpen]);

    return (
        <Dialog open={isOpen} onOpenChange={setIsOpen}>
            <DialogTrigger asChild>
                <Button size="sm" variant="outline">
                    <Wand2 className="mr-2 h-4 w-4" />
                    Select Ideas
                </Button>
            </DialogTrigger>
            <DialogContent className="max-w-4xl h-[80vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle>Select Ideas for the Session</DialogTitle>
                    <DialogDescription>
                        You can manually select ideas or use the lottery to pick random ones.
                    </DialogDescription>
                </DialogHeader>
                <div className="flex-1 overflow-y-auto pr-2 -mr-6">
                    {availableIdeas.length > 0 ? (
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-1">
                            {availableIdeas.map(idea => (
                                <Card
                                    key={idea.ideaId}
                                    onClick={() => handleToggleSelection(idea.ideaId)}
                                    className={cn(
                                        "cursor-pointer transition-all duration-300 relative overflow-hidden",
                                        selectedIdeas.includes(idea.ideaId) && !isPicking && "ring-2 ring-primary ring-offset-2",
                                        spinning && !finalSelection.includes(idea.ideaId) && "opacity-50 scale-95",
                                        finalSelection.includes(idea.ideaId) && "bg-green-100 dark:bg-green-900/50 ring-2 ring-green-500 animate-pulse",
                                        isLoading && "cursor-not-allowed"
                                    )}
                                >
                                    <CardContent className="p-4">
                                        <CardTitle className="text-base font-bold line-clamp-2">{idea.title}</CardTitle>
                                        <CardDescription className="text-xs line-clamp-3 mt-2">{idea.description}</CardDescription>
                                        {selectedIdeas.includes(idea.ideaId) && !isPicking && (
                                            <div className="absolute top-2 right-2 h-5 w-5 bg-primary rounded-full flex items-center justify-center">
                                                <CheckCircle className="h-5 w-5 text-primary-foreground" />
                                            </div>
                                        )}
                                    </CardContent>
                                </Card>
                            ))}
                        </div>
                    ) : (
                         <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
                            <p className="font-semibold">No submitted ideas available.</p>
                            <p className="text-sm">New ideas with a 'submitted' status will appear here.</p>
                        </div>
                    )}
                </div>
                <DialogFooter>
                    <Button variant="ghost" onClick={handlePickRandomly} disabled={isLoading || availableIdeas.length === 0}>
                        <Ticket className={`mr-2 h-4 w-4 ${isPicking ? 'animate-spin' : ''}`} />
                        {isPicking ? 'Picking...' : 'Start Lottery'}
                    </Button>
                    <Button onClick={handleConfirmSelection} disabled={isLoading || selectedIdeas.length === 0}>
                        Confirm {selectedIdeas.length > 0 ? `(${selectedIdeas.length})` : ''} Selection
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
