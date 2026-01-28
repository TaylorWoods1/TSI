import Image from 'next/image';
import { placeHolderImages } from '@/lib/placeholder-images';

export default function PublicLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen w-full flex-col items-center justify-center bg-background p-4">
      <div className="mb-8 flex items-center gap-3 text-2xl font-bold text-primary">
         <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="h-8 w-8"
        >
          <path d="M15.5 3.84a1 1 0 0 1 1.28-1.28l4.33 4.33a1 1 0 0 1-1.28 1.28L15.5 3.84z" />
          <path d="M12.5 6.84a1 1 0 0 0-1.28-1.28l-4.33 4.33a1 1 0 0 0 1.28 1.28L12.5 6.84z" />
          <path d="m14 17 3-3" />
          <path d="M10 17v-3.5" />
          <path d="M7 11v1.5" />
          <path d="m7 17 3-3" />
          <path d="M17 14h-1.5" />
          <circle cx="12" cy="12" r="10" />
        </svg>
        Tech Sol Innovations
      </div>
      {children}
    </div>
  );
}
