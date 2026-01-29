/**
 * @fileoverview Defines server actions for the application, which handle data mutations
 * and business logic on the server side.
 */
'use server';

import { PickRandomIdeaInput, PickRandomIdeaOutput } from '@/ai/flows/random-idea-selection';
import { mockIdeas, mockSessions } from '@/lib/data';
import { revalidatePath } from 'next/cache';

/**
 * Picks a single random idea with 'submitted' status, adds it to the specified session,
 * and updates the state of both the idea and the session.
 *
 * This function simulates a database transaction by:
 * 1. Filtering for available ideas.
 * 2. Randomly selecting one idea.
 * 3. Updating the idea's status to 'selectedForSession'.
 * 4. Adding the idea's ID to the session's list of selected ideas.
 * 5. Changing the session's status to 'active' if it was 'planned'.
 * 6. Revalidating relevant page paths to refresh the UI.
 *
 * @param input - Contains the sessionId for which to pick an idea.
 * @returns A promise that resolves to an object containing the picked idea's ID, or null if no ideas were available.
 */
export async function pickAndSelectRandomIdeaForSessionAction(input: PickRandomIdeaInput): Promise<PickRandomIdeaOutput> {
  const submittedIdeas = mockIdeas.filter(idea => idea.status === 'submitted');
  
  if (submittedIdeas.length === 0) {
    return { ideaId: null };
  }
  
  // Mocking the random selection
  const randomIndex = Math.floor(Math.random() * submittedIdeas.length);
  const randomIdea = submittedIdeas[randomIndex];
  const pickedIdeaId = randomIdea.ideaId;

  // --- Update state (simulating database updates) ---
  const sessionIndex = mockSessions.findIndex(s => s.sessionId === input.sessionId);

  if (sessionIndex !== -1) {
    // Add the new idea ID to the session's list if it's not already there.
    if (!mockSessions[sessionIndex].selectedIdeaIds.includes(pickedIdeaId)) {
        mockSessions[sessionIndex].selectedIdeaIds.push(pickedIdeaId);
    }
    
    // Update the idea's status to 'selectedForSession'.
    const ideaIndex = mockIdeas.findIndex(i => i.ideaId === pickedIdeaId);
    if (ideaIndex !== -1) {
      mockIdeas[ideaIndex].status = 'selectedForSession';
    }
    
    // If the session was 'planned', move it to 'active'.
    if (mockSessions[sessionIndex].status === 'planned') {
        mockSessions[sessionIndex].status = 'active';
    }

    console.log(`Selected idea ${pickedIdeaId} for session ${input.sessionId}`);
  } else {
    console.error(`Session ${input.sessionId} not found.`);
    return { ideaId: null };
  }
  
  // Revalidate paths to reflect changes in the UI.
  revalidatePath(`/sessions/${input.sessionId}`);
  revalidatePath('/admin/sessions');
  revalidatePath('/admin/ideas');
  revalidatePath('/dashboard');

  return { ideaId: pickedIdeaId };
}
