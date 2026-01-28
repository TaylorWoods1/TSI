'use server';

import { pickRandomIdea, PickRandomIdeaInput, PickRandomIdeaOutput } from '@/ai/flows/random-idea-selection';
import { mockIdeas } from '@/lib/data';
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
