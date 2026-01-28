'use server';

/**
 * @fileOverview Randomly selects an idea from the pool of submitted ideas for a given ideation session.
 *
 * - pickRandomIdea - A function that handles the random idea selection process.
 * - PickRandomIdeaInput - The input type for the pickRandomIdea function.
 * - PickRandomIdeaOutput - The return type for the pickRandomIdea function.
 */

import {ai} from '@/ai/genkit';
import {z} from 'genkit';

const PickRandomIdeaInputSchema = z.object({
  sessionId: z
    .string()
    .uuid()
    .describe('The ID of the ideation session for which to select an idea.'),
});
export type PickRandomIdeaInput = z.infer<typeof PickRandomIdeaInputSchema>;

const PickRandomIdeaOutputSchema = z.object({
  ideaId: z
    .string()
    .uuid()
    .nullable()
    .describe('The ID of the randomly selected idea, or null if no idea was selected.'),
});
export type PickRandomIdeaOutput = z.infer<typeof PickRandomIdeaOutputSchema>;

export async function pickRandomIdea(input: PickRandomIdeaInput): Promise<PickRandomIdeaOutput> {
  return pickRandomIdeaFlow(input);
}

const pickRandomIdeaPrompt = ai.definePrompt({
  name: 'pickRandomIdeaPrompt',
  input: {schema: PickRandomIdeaInputSchema},
  output: {schema: PickRandomIdeaOutputSchema},
  prompt: `You are a helpful assistant tasked with selecting a random idea for an ideation session.

  Given the session ID: {{{sessionId}}}, your task is to:
  1.  Query the database for all ideas that have a status of 'submitted'.
  2.  Randomly select one idea from the list of submitted ideas.
  3.  Return the ideaId of the selected idea. If no ideas are submitted, return null.
  
  Ensure that the outputted ideaId is a valid UUID.
  `,
});

const pickRandomIdeaFlow = ai.defineFlow(
  {
    name: 'pickRandomIdeaFlow',
    inputSchema: PickRandomIdeaInputSchema,
    outputSchema: PickRandomIdeaOutputSchema,
  },
  async input => {
    const {output} = await pickRandomIdeaPrompt(input);
    return output!;
  }
);
