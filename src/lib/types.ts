/**
 * @fileoverview Centralized type definitions for the application's data models.
 */

/**
 * Represents a user account in the system.
 */
export type User = {
  /** A unique identifier for the user. */
  userId: string;
  /** The user's email address. */
  email: string;
  /** The user's first name. */
  firstName: string;
  /** The user's last name. */
  lastName: string;
  /** The user's role, determining their permissions. */
  role: 'employee' | 'administrator';
  /** The ISO date string of when the user account was created. */
  createdAt: string;
};

/**
 * Represents an idea submitted by a user.
 */
export type Idea = {
  /** A unique identifier for the idea. */
  ideaId: string;
  /** The ID of the user who submitted the idea, or null if anonymous. */
  userId: string | null;
  /** A short, descriptive title for the idea. */
  title: string;
  /** A detailed description of the idea. */
  description: string;
  /** A flag indicating if the submission is anonymous. */
  isAnonymous: boolean;
  /** The current status of the idea in the workflow. */
  status: 'submitted' | 'selectedForSession' | 'archived';
  /** The ISO date string of when the idea was submitted. */
  submissionDate: string;
};

/**
 * Represents a collaborative session for workshopping ideas.
 */
export type IdeationSession = {
  /** A unique identifier for the session. */
  sessionId: string;
  /** The name or title of the session. */
  name: string;
  /** A brief description of the session's goals. */
  description: string;
  /** The ISO date string for when the session is scheduled. */
  sessionDate: string;
  /** A list of idea IDs that have been selected for this session. */
  selectedIdeaIds: string[];
  /** The current status of the session. */
  status: 'planned' | 'active' | 'completed';
};

/**
 * Represents a specific use case derived from an idea within a session.
 */
export type UseCase = {
  /** A unique identifier for the use case. */
  useCaseId: string;
  /** The ID of the session this use case belongs to. */
  sessionId: string;
  /** The ID of the parent idea for this use case. */
  ideaId: string;
  /** A description of the use case. */
  description:string;
  /** The ISO date string of when the use case was created. */
  createdAt: string;
};

/**
 * Represents a potential solution or implementation for a given use case.
 */
export type Solution = {
  /** A unique identifier for the solution. */
  solutionId: string;
  /** The ID of the session this solution belongs to. */
  sessionId: string;
  /** The ID of the parent idea for this solution. */
  ideaId: string;
  /** The ID of the use case this solution addresses. */
  useCaseId: string;
  /** A description of the solution. */
  description: string;
  /** The ISO date string of when the solution was created. */
  createdAt: string;
};
