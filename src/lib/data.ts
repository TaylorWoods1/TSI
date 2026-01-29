/**
 * @fileoverview Provides mock data for the application.
 * In a real-world scenario, this data would be fetched from a database or API.
 * This file serves as a placeholder for prototyping and UI development.
 */
import type { User, Idea, IdeationSession, UseCase, Solution } from './types';

/** A mock user with administrator privileges. */
export const mockAdmin: User = {
  userId: 'a1b2c3d4-e5f6-7890-1234-567890abcdef',
  email: 'admin@tsi.com',
  firstName: 'Admin',
  lastName: 'User',
  role: 'administrator',
  createdAt: new Date().toISOString(),
};

/** A list of mock users for the platform. */
export const mockUsers: User[] = [
  mockAdmin,
  {
    userId: 'b2c3d4e5-f6a7-8901-2345-67890abcdef1',
    email: 'employee1@tsi.com',
    firstName: 'John',
    lastName: 'Doe',
    role: 'employee',
    createdAt: new Date('2023-01-15').toISOString(),
  },
  {
    userId: 'c3d4e5f6-a7b8-9012-3456-7890abcdef12',
    email: 'employee2@tsi.com',
    firstName: 'Jane',
    lastName: 'Smith',
    role: 'employee',
    createdAt: new Date('2023-02-20').toISOString(),
  },
  {
    userId: 'd4e5f6a7-b8c9-0123-4567-890abcdef123',
    email: 'employee3@tsi.com',
    firstName: 'Peter',
    lastName: 'Jones',
    role: 'employee',
    createdAt: new Date('2023-03-10').toISOString(),
  },
];

/** A list of mock ideas submitted by users. */
export const mockIdeas: Idea[] = [
  {
    ideaId: 'idea-001',
    userId: mockUsers[1].userId,
    title: 'AI-Powered Customer Support Chatbot',
    description: 'Implement a chatbot to handle common customer queries, reducing support team workload.',
    isAnonymous: false,
    status: 'selectedForSession',
    submissionDate: new Date('2023-10-01').toISOString(),
  },
  {
    ideaId: 'idea-002',
    userId: mockUsers[2].userId,
    title: 'Gamified Employee Onboarding Process',
    description: 'Create an interactive and engaging onboarding experience for new hires.',
    isAnonymous: false,
    status: 'submitted',
    submissionDate: new Date('2023-10-05').toISOString(),
  },
  {
    ideaId: 'idea-003',
    userId: null,
    title: 'Internal Tool for Booking Meeting Rooms',
    description: 'A simple web app to see room availability and book slots.',
    isAnonymous: true,
    status: 'submitted',
    submissionDate: new Date('2023-10-10').toISOString(),
  },
  {
    ideaId: 'idea-004',
    userId: mockUsers[3].userId,
    title: 'Sustainable Office Initiatives',
    description: 'Ideas for reducing waste and energy consumption in the office, like smart lighting.',
    isAnonymous: false,
    status: 'archived',
    submissionDate: new Date('2023-09-20').toISOString(),
  },
  {
    ideaId: 'idea-005',
    userId: mockUsers[1].userId,
    title: 'Company-wide Mentorship Program',
    description: 'A platform to connect junior and senior employees for mentorship opportunities.',
    isAnonymous: false,
    status: 'submitted',
    submissionDate: new Date('2023-10-12').toISOString(),
  },
];

/** A list of mock ideation sessions. */
export const mockSessions: IdeationSession[] = [
  {
    sessionId: 'session-01',
    name: 'Q4 Innovation Sprint - Customer Experience',
    description: 'Focus on ideas that will improve our customer journey and satisfaction.',
    sessionDate: new Date(new Date().setDate(new Date().getDate() + 10)).toISOString(),
    selectedIdeaIds: ['idea-001'],
    status: 'active',
  },
  {
    sessionId: 'session-02',
    name: 'Q1 2024 Employee Wellness',
    description: 'Brainstorming session for initiatives related to employee health and wellness.',
    sessionDate: new Date(new Date().setDate(new Date().getDate() + 30)).toISOString(),
    selectedIdeaIds: [],
    status: 'planned',
  },
  {
    sessionId: 'session-03',
    name: 'Q3 Internal Process Optimization',
    description: 'A look back at ideas for improving our internal workflows.',
    sessionDate: new Date('2023-09-15').toISOString(),
    selectedIdeaIds: ['idea-004'],
    status: 'completed',
  },
];

/** A list of mock use cases associated with ideas in sessions. */
export const mockUseCases: UseCase[] = [
  {
    useCaseId: 'uc-001',
    sessionId: 'session-01',
    ideaId: 'idea-001',
    description: 'Automated response to "Where is my order?" queries.',
    createdAt: new Date().toISOString(),
  },
  {
    useCaseId: 'uc-002',
    sessionId: 'session-01',
    ideaId: 'idea-001',
    description: 'Triage complex support tickets to the correct department.',
    createdAt: new Date().toISOString(),
  },
];

/** A list of mock solutions for specific use cases. */
export const mockSolutions: Solution[] = [
  {
    solutionId: 'sol-001',
    sessionId: 'session-01',
    ideaId: 'idea-001',
    useCaseId: 'uc-001',
    description: 'Integrate with shipping provider API to give real-time order status.',
    createdAt: new Date().toISOString(),
  },
  {
    solutionId: 'sol-002',
    sessionId: 'session-01',
    ideaId: 'idea-001',
    useCaseId: 'uc-001',
    description: 'Use a decision tree based on order history to provide accurate info.',
    createdAt: new Date().toISOString(),
  },
];
