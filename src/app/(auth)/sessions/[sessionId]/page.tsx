import { PageHeader } from '@/components/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { mockIdeas, mockSessions, mockUseCases, mockSolutions } from '@/lib/data';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Lightbulb, Plus, Workflow, Sparkles, AlertCircle } from 'lucide-react';

export default function SessionDetailPage({ params }: { params: { sessionId: string } }) {
  const session = mockSessions.find((s) => s.sessionId === params.sessionId);
  const selectedIdeas = mockIdeas.filter((i) => session?.selectedIdeaIds.includes(i.ideaId));
  const useCases = mockUseCases.filter((uc) => uc.sessionId === session?.sessionId);
  const solutions = mockSolutions.filter((s) => s.sessionId === session?.sessionId);

  if (!session) {
    return (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          <AlertCircle className="mr-2 h-5 w-5" /> Session not found.
        </div>
    );
  }

  return (
    <div className="container mx-auto p-0">
      <PageHeader title={session.name} description={session.description} />

      {selectedIdeas.length > 0 ? (
        <div className="grid gap-8 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <Card className="mb-6">
                <CardHeader>
                    <CardTitle className="flex items-center gap-3">
                        <Lightbulb className="h-6 w-6 text-primary"/>
                        Selected Ideas
                    </CardTitle>
                    <CardDescription>The focus of this ideation session.</CardDescription>
                </CardHeader>
              <CardContent className="space-y-4">
                {selectedIdeas.map(idea => (
                     <div key={idea.ideaId} className="rounded-lg border bg-card p-4">
                        <h3 className="text-xl font-bold">{idea.title}</h3>
                        <p className="mt-2 text-muted-foreground">{idea.description}</p>
                    </div>
                ))}
              </CardContent>
            </Card>

            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold flex items-center gap-2"><Workflow className="h-6 w-6 text-primary" /> Use Cases</h2>
                 <p className="text-muted-foreground">What are the specific applications or scenarios for these ideas?</p>
              </div>
              
              <div className="space-y-4">
                {useCases.map((uc) => (
                  <Card key={uc.useCaseId}>
                    <CardContent className="pt-6">
                      <p>{uc.description}</p>
                    </CardContent>
                  </Card>
                ))}
                {session.status === 'active' && (
                    <Card className="border-dashed">
                        <CardHeader>
                            <CardTitle className="text-lg">Add a New Use Case</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <form className="space-y-4">
                                <Textarea placeholder="Describe a new use case..."/>
                                <Button>
                                    <Plus className="mr-2 h-4 w-4" />
                                    Submit Use Case
                                </Button>
                            </form>
                        </CardContent>
                    </Card>
                )}
              </div>

             <div className="pt-6">
                <h2 className="text-2xl font-bold flex items-center gap-2"><Sparkles className="h-6 w-6 text-primary" /> Solutions</h2>
                 <p className="text-muted-foreground">How can we implement these use cases?</p>
              </div>

               <div className="space-y-4">
                {solutions.map((sol) => (
                  <Card key={sol.solutionId}>
                    <CardHeader>
                        <CardDescription>
                            For Use Case: {useCases.find(uc => uc.useCaseId === sol.useCaseId)?.description.substring(0, 50)}...
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <p>{sol.description}</p>
                    </CardContent>
                  </Card>
                ))}
                {session.status === 'active' && (
                     <Card className="border-dashed">
                        <CardHeader>
                            <CardTitle className="text-lg">Propose a New Solution</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <form className="space-y-4">
                                <Textarea placeholder="Describe a new solution..."/>
                                <Button>
                                    <Plus className="mr-2 h-4 w-4" />
                                    Submit Solution
                                </Button>
                            </form>
                        </CardContent>
                    </Card>
                )}
              </div>

            </div>
          </div>
        </div>
      ) : (
        <Alert>
            <Lightbulb className="h-4 w-4" />
            <AlertTitle>Idea Selection Pending</AlertTitle>
            <AlertDescription>
            An idea for this session has not been selected yet. Check back soon!
            {session.status === 'planned' && " An administrator can select ideas from the Session Management page."}
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}
