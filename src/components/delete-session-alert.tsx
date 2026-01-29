/**
 * @fileoverview Defines a reusable alert dialog for confirming the deletion of a session.
 */
'use client';

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import type { IdeationSession } from '@/lib/types';

/**
 * Props for the DeleteSessionAlert component.
 */
interface DeleteSessionAlertProps {
  /** The session object to be deleted. Used to display the session name. */
  session: IdeationSession | null;
  /** Controls whether the dialog is open. */
  isOpen: boolean;
  /** Function to call when the dialog should be closed (e.g., on cancel or escape). */
  onClose: () => void;
  /** Function to call when the user confirms the deletion. */
  onConfirm: () => void;
}

/**
 * A modal dialog that asks the user to confirm the deletion of an ideation session.
 * It is controlled by the `isOpen` prop and uses callbacks to handle user actions.
 *
 * @param props - The properties for the component.
 * @returns A JSX element representing the confirmation dialog.
 */
export function DeleteSessionAlert({ session, isOpen, onClose, onConfirm }: DeleteSessionAlertProps) {

  const handleConfirm = () => {
    onConfirm();
    onClose();
  }

  // This handler ensures that the dialog correctly signals to close when interacted with.
  const handleOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
    }
  };

  return (
    <AlertDialog open={isOpen} onOpenChange={handleOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete the "{session?.name}" session.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={handleConfirm} variant="destructive">
            Yes, delete it
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
