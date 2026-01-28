'use server';

import { pickRandomIdea, PickRandomIdeaInput, PickRandomIdeaOutput } from '@/ai/flows/random-idea-selection';
import { mockIdeas, mockSessions } from '@/lib/data';
import { revalidatePath } from 'next/cache';

export async function pickRandomIdeaAction(input: PickRandomIdeaInput): Promise<PickRandomIdeaOutput> {
  // In a real scenario, the GenAI flow would query a database.
  // Here, we simulate it by checking our mock data.
  const submittedIdeas = mockIdeas.filter(idea => idea.status === 'submitted');
  
  if (submittedIdeas.length === 0) {
    return { ideaId: null };
  }

  // The Genkit flow is defined to do this, but we'll mock the logic for a predictable result.
  // const result = await pickRandomIdea(input);
  
  const randomIndex = Math.floor(Math.random() * submittedIdeas.length);
  const randomIdea = submittedIdeas[randomIndex];
  
  const result: PickRandomIdeaOutput = {
      ideaId: randomIdea.ideaId
  }

  // In a real app, we'd update the database here.
  // e.g., db.updateIdeaStatus(result.ideaId, 'selectedForSession');
  // and db.updateSession(input.sessionId, { selectedIdeaId: result.ideaId });
  
  console.log(`Picked idea ${result.ideaId} for session ${input.sessionId}`);
  
  revalidatePath('/admin/sessions');
  return result;
}


export type SelectIdeasForSessionInput = {
    sessionId: string;
    ideaIds: string[];
};

export async function selectIdeasForSessionAction(input: SelectIdeasForSessionInput): Promise<{ success: boolean }> {
  // In a real app, you would update the database here.
  // We'll simulate this by finding the session in our mock data and updating it.
  const sessionIndex = mockSessions.findIndex(s => s.sessionId === input.sessionId);

  if (sessionIndex !== -1) {
    mockSessions[sessionIndex].selectedIdeaIds = input.ideaIds;
    // Also update the status of the selected ideas to 'selectedForSession'
    input.ideaIds.forEach(ideaId => {
      const ideaIndex = mockIdeas.findIndex(i => i.ideaId === ideaId);
      if (ideaIndex !== -1) {
        mockIdeas[ideaIndex].status = 'selectedForSession';
      }
    });
    
    // Also update session status from 'planned' to 'active' if it was planned and ideas were selected
    if (mockSessions[sessionIndex].status === 'planned' && input.ideaIds.length > 0) {
        mockSessions[sessionIndex].status = 'active';
    }

    console.log(`Selected ideas ${input.ideaIds.join(', ')} for session ${input.sessionId}`);
  } else {
    console.error(`Session ${input.sessionId} not found.`);
    return { success: false };
  }
  
  revalidatePath('/admin/sessions');
  revalidatePath(`/sessions/${input.sessionId}`);
  revalidatePath('/admin/ideas');
  return { success: true };
}
