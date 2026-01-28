export type User = {
  userId: string;
  email: string;
  firstName: string;
  lastName: string;
  role: 'employee' | 'administrator';
  createdAt: string;
};

export type Idea = {
  ideaId: string;
  userId: string | null;
  title: string;
  description: string;
  isAnonymous: boolean;
  status: 'submitted' | 'selectedForSession' | 'archived';
  submissionDate: string;
};

export type IdeationSession = {
  sessionId: string;
  name: string;
  description: string;
  sessionDate: string;
  selectedIdeaId: string | null;
  status: 'planned' | 'active' | 'completed';
};

export type UseCase = {
  useCaseId: string;
  sessionId: string;
  ideaId: string;
  description: string;
  createdAt: string;
};

export type Solution = {
  solutionId: string;
  sessionId: string;
  ideaId: string;
  useCaseId: string;
  description: string;
  createdAt: string;
};
