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

interface DeleteSessionAlertProps {
  session: IdeationSession | null;
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export function DeleteSessionAlert({ session, isOpen, onClose, onConfirm }: DeleteSessionAlertProps) {

  const handleConfirm = () => {
    onConfirm();
    onClose();
  }

  return (
    <AlertDialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete the "{session?.name}" session.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} variant="destructive">
            Yes, delete it
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
