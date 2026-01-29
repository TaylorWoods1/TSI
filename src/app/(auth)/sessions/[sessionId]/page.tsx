'use client';

import {
  AlertCircle,
  Edit,
  Lightbulb,
  MoreVertical,
  Plus,
  Sparkles,
  Trash2,
  Workflow,
} from 'lucide-react';
import { useParams } from 'next/navigation';

import { PageHeader } from '@/components/page-header';
import { SessionIdeaPicker } from '@/components/session-idea-picker';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { useAuth } from '@/lib/auth';
import {
  mockIdeas,
  mockSessions,
  mockSolutions,
  mockUseCases,
} from '@/lib/data';

export default function SessionDetailPage() {
  const params = useParams<{ sessionId: string }>();
  const { user } = useAuth();
  const session = mockSessions.find((s) => s.sessionId === params.sessionId);
  const selectedIdeas = mockIdeas.filter((i) =>
    session?.selectedIdeaIds.includes(i.ideaId)
  );
  const submittedIdeasCount = mockIdeas.filter(
    (i) => i.status === 'submitted'
  ).length;

  const isAdmin = user?.role === 'administrator';
  const isSessionActive = session?.status === 'active';
  const isSessionCompleted = session?.status === 'completed';

  if (!session) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <AlertCircle className="mr-2 h-5 w-5" /> Session not found.
      </div>
    );
  }

  // A component for the placeholder when no ideas are selected yet.
  const InitialPicker = () => (
    <Card className="text-center">
      <CardHeader>
        <CardTitle className="flex items-center justify-center gap-3">
          <Lightbulb className="h-6 w-6 text-primary" />
          Idea Selection
        </CardTitle>
        <CardDescription>
          This session hasn't started yet. Once an administrator selects the
          first idea, the workshopping can begin.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-lg font-semibold">{submittedIdeasCount}</p>
        <p className="text-sm text-muted-foreground">
          Submitted ideas available
        </p>
      </CardContent>
      {isAdmin && !isSessionCompleted && (
        <CardFooter className="flex-col gap-4 border-t pt-6">
          <SessionIdeaPicker
            sessionId={session.sessionId}
            buttonText="Pick First Idea"
          />
          <p className="text-xs text-muted-foreground">
            This will randomly select one submitted idea and start the session.
          </p>
        </CardFooter>
      )}
    </Card>
  );

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title={session.name}
        description={`${session.description} - ${new Date(
          session.sessionDate
        ).toLocaleDateString('en-US', {
          year: 'numeric',
          month: 'long',
          day: 'numeric',
        })}`}
      >
        {isAdmin && isSessionActive && (
          <SessionIdeaPicker
            sessionId={session.sessionId}
            buttonText="Pick Next Idea"
            variant="outline"
          />
        )}
      </PageHeader>

      {selectedIdeas.length > 0 ? (
        <Tabs
          defaultValue={selectedIdeas[selectedIdeas.length - 1].ideaId}
          className="w-full"
        >
          <TabsList className="mb-6 h-auto justify-start">
            {selectedIdeas.map((idea) => (
              <TabsTrigger key={idea.ideaId} value={idea.ideaId}>
                {idea.title}
              </TabsTrigger>
            ))}
          </TabsList>

          {selectedIdeas.map((idea) => {
            const ideaUseCases = mockUseCases.filter(
              (uc) =>
                uc.ideaId === idea.ideaId && uc.sessionId === session.sessionId
            );
            const ideaSolutions = mockSolutions.filter(
              (sol) =>
                sol.ideaId === idea.ideaId && sol.sessionId === session.sessionId
            );

            return (
              <TabsContent
                key={idea.ideaId}
                value={idea.ideaId}
                className="mt-0"
              >
                <div className="grid gap-8 md:grid-cols-3">
                  {/* Left Column: Idea Details */}
                  <div className="md:col-span-1">
                    <div className="sticky top-20 space-y-6">
                      <Card>
                        <CardHeader>
                          <CardTitle className="flex items-center gap-3">
                            <Lightbulb className="h-6 w-6 text-primary" />
                            Selected Idea
                          </CardTitle>
                          <CardDescription>
                            The focus of this ideation tab.
                          </CardDescription>
                        </CardHeader>
                        <CardContent>
                          <div className="rounded-lg border bg-background p-4">
                            <h3 className="text-lg font-bold">{idea.title}</h3>
                            <p className="mt-1 text-sm text-muted-foreground">
                              {idea.description}
                            </p>
                          </div>
                        </CardContent>
                      </Card>
                    </div>
                  </div>

                  {/* Right Column: Use Cases & Solutions */}
                  <div className="space-y-8 md:col-span-2">
                    {/* Use Case Development Area */}
                    <section>
                      <div className="mb-6">
                        <h2 className="flex items-center gap-3 text-2xl font-bold">
                          <Workflow className="h-6 w-6 text-primary" /> Use
                          Cases
                        </h2>
                        <p className="mt-1 text-muted-foreground">
                          Specific applications for "{idea.title}"
                        </p>
                      </div>
                      <div className="space-y-4">
                        {ideaUseCases.map((uc) => (
                          <Card key={uc.useCaseId}>
                            <CardContent className="flex items-start justify-between gap-4 p-4">
                              <p className="flex-1 pt-1">{uc.description}</p>
                              {!isSessionCompleted && (
                                <DropdownMenu>
                                  <DropdownMenuTrigger asChild>
                                    <Button
                                      variant="ghost"
                                      size="icon"
                                      className="h-8 w-8 shrink-0"
                                    >
                                      <MoreVertical className="h-4 w-4" />
                                    </Button>
                                  </DropdownMenuTrigger>
                                  <DropdownMenuContent align="end">
                                    <DropdownMenuItem>
                                      <Edit /> Edit
                                    </DropdownMenuItem>
                                    <DropdownMenuItem className="text-destructive focus:text-destructive">
                                      <Trash2 /> Delete
                                    </DropdownMenuItem>
                                  </DropdownMenuContent>
                                </DropdownMenu>
                              )}
                            </CardContent>
                          </Card>
                        ))}

                        {!isSessionCompleted && (
                          <Card className="border-dashed bg-muted/50">
                            <CardHeader>
                              <CardTitle className="text-lg font-semibold">
                                Add a New Use Case
                              </CardTitle>
                              <CardDescription>
                                Contribute a new application for this idea.
                              </CardDescription>
                            </CardHeader>
                            <CardContent>
                              <form className="space-y-4">
                                <Textarea placeholder="Describe the new use case..." />
                                <Button>
                                  <Plus className="mr-2" />
                                  Submit Use Case
                                </Button>
                              </form>
                            </CardContent>
                          </Card>
                        )}
                      </div>
                    </section>

                    <Separator />

                    {/* Solution Development Area */}
                    <section>
                      <div className="mb-6">
                        <h2 className="flex items-center gap-3 text-2xl font-bold">
                          <Sparkles className="h-6 w-6 text-primary" />{' '}
                          Solutions
                        </h2>
                        <p className="mt-1 text-muted-foreground">
                          How can we implement these use cases?
                        </p>
                      </div>
                      <div className="space-y-4">
                        {ideaSolutions.map((sol) => (
                          <Card key={sol.solutionId}>
                            <CardHeader className="p-4 pb-2">
                              <CardDescription>
                                For Use Case:{' '}
                                <span className="font-medium text-foreground">
                                  "
                                  {
                                    ideaUseCases.find(
                                      (uc) => uc.useCaseId === sol.useCaseId
                                    )?.description
                                  }
                                  "
                                </span>
                              </CardDescription>
                            </CardHeader>
                            <CardContent className="flex items-start justify-between gap-4 p-4 pt-0">
                              <p className="flex-1 pt-1">{sol.description}</p>
                              {!isSessionCompleted && (
                                <DropdownMenu>
                                  <DropdownMenuTrigger asChild>
                                    <Button
                                      variant="ghost"
                                      size="icon"
                                      className="h-8 w-8 shrink-0"
                                    >
                                      <MoreVertical className="h-4 w-4" />
                                    </Button>
                                  </DropdownMenuTrigger>
                                  <DropdownMenuContent align="end">
                                    <DropdownMenuItem>
                                      <Edit /> Edit
                                    </DropdownMenuItem>
                                    <DropdownMenuItem className="text-destructive focus:text-destructive">
                                      <Trash2 /> Delete
                                    </DropdownMenuItem>
                                  </DropdownMenuContent>
                                </DropdownMenu>
                              )}
                            </CardContent>
                          </Card>
                        ))}

                        {!isSessionCompleted && ideaUseCases.length > 0 && (
                          <Card className="border-dashed bg-muted/50">
                            <CardHeader>
                              <CardTitle className="text-lg font-semibold">
                                Propose a New Solution
                              </CardTitle>
                              <CardDescription>
                                Describe how to implement a specific use case.
                              </CardDescription>
                            </CardHeader>
                            <CardContent>
                              <form className="space-y-4">
                                <div className="grid gap-2">
                                  <Label
                                    htmlFor={`use-case-select-${idea.ideaId}`}
                                  >
                                    Link to Use Case
                                  </Label>
                                  <Select>
                                    <SelectTrigger
                                      id={`use-case-select-${idea.ideaId}`}
                                    >
                                      <SelectValue placeholder="Select a use case" />
                                    </SelectTrigger>
                                    <SelectContent>
                                      {ideaUseCases.map((uc) => (
                                        <SelectItem
                                          key={uc.useCaseId}
                                          value={uc.useCaseId}
                                        >
                                          {uc.description}
                                        </SelectItem>
                                      ))}
                                    </SelectContent>
                                  </Select>
                                </div>
                                <div className="grid gap-2">
                                  <Label
                                    htmlFor={`solution-description-${idea.ideaId}`}
                                  >
                                    Solution Description
                                  </Label>
                                  <Textarea
                                    id={`solution-description-${idea.ideaId}`}
                                    placeholder="Describe the new solution..."
                                  />
                                </div>
                                <Button>
                                  <Plus className="mr-2" />
                                  Submit Solution
                                </Button>
                              </form>
                            </CardContent>
                          </Card>
                        )}
                      </div>
                    </section>
                  </div>
                </div>
              </TabsContent>
            );
          })}
        </Tabs>
      ) : isSessionCompleted ? (
        <Alert>
          <Lightbulb className="h-4 w-4" />
          <AlertTitle>Session Completed</AlertTitle>
          <AlertDescription>
            This session is over. No ideas were selected.
          </AlertDescription>
        </Alert>
      ) : (
        <InitialPicker />
      )}
    </div>
  );
}
