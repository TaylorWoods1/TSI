'use client';

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useAuth } from '@/lib/auth';
import {
  Home,
  Lightbulb,
  Presentation,
  Settings,
  UserCog,
  BrainCircuit,
  Boxes,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

const employeeNavItems = [
  { href: '/dashboard', icon: Home, label: 'Dashboard' },
  { href: '/ideas/submit', icon: Lightbulb, label: 'Submit Idea' },
  { href: '/sessions', icon: Presentation, label: 'Sessions' },
];

const adminNavItems = [
  { href: '/admin/ideas', icon: Boxes, label: 'Manage Ideas' },
  { href: '/admin/sessions', icon: BrainCircuit, label: 'Manage Sessions' },
  { href: '/admin/users', icon: UserCog, label: 'Manage Users' },
];

export function AppSidebar() {
  const { user } = useAuth();
  const pathname = usePathname();

  const navItems = user?.role === 'administrator' ? [...employeeNavItems, ...adminNavItems] : employeeNavItems;

  return (
    <aside className="fixed inset-y-0 left-0 z-10 hidden w-14 flex-col border-r bg-background sm:flex">
      <TooltipProvider>
        <nav className="flex flex-col items-center gap-4 px-2 py-4">
          <Link
            href="/dashboard"
            className="group flex h-9 w-9 shrink-0 items-center justify-center gap-2 rounded-full bg-primary text-lg font-semibold text-primary-foreground md:h-8 md:w-8 md:text-base"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="h-4 w-4 transition-all group-hover:scale-110"
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
            <span className="sr-only">Taylor Inc</span>
          </Link>

          {navItems.map((item) => (
            <Tooltip key={item.href}>
              <TooltipTrigger asChild>
                <Link
                  href={item.href}
                  className={`flex h-9 w-9 items-center justify-center rounded-lg transition-colors md:h-8 md:w-8 ${
                    pathname.startsWith(item.href) && item.href !== '/dashboard' || pathname === item.href
                      ? 'bg-accent text-accent-foreground'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                >
                  <item.icon className="h-5 w-5" />
                  <span className="sr-only">{item.label}</span>
                </Link>
              </TooltipTrigger>
              <TooltipContent side="right">{item.label}</TooltipContent>
            </Tooltip>
          ))}
        </nav>
        <nav className="mt-auto flex flex-col items-center gap-4 px-2 py-4">
          <Tooltip>
            <TooltipTrigger asChild>
              <Link
                href="#"
                className="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:text-foreground md:h-8 md:w-8"
              >
                <Settings className="h-5 w-5" />
                <span className="sr-only">Settings</span>
              </Link>
            </TooltipTrigger>
            <TooltipContent side="right">Settings</TooltipContent>
          </Tooltip>
        </nav>
      </TooltipProvider>
    </aside>
  );
}
