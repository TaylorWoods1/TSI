/**
 * @fileoverview Defines the main application sidebar, providing navigation and logout functionality.
 */
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
  UserCog,
  Boxes,
  LogOut,
} from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { LogoIcon } from './icons/logo-icon';

// Navigation items for standard employees
const employeeNavItems = [
  { href: '/dashboard', icon: Home, label: 'Dashboard' },
  { href: '/ideas/submit', icon: Lightbulb, label: 'Submit Idea' },
  { href: '/sessions', icon: Presentation, label: 'Sessions' },
];

// Additional navigation items for administrators
const adminNavItems = [
  { href: '/admin/ideas', icon: Boxes, label: 'Manage Ideas' },
  { href: '/admin/users', icon: UserCog, label: 'Manage Users' },
];

/**
 * The main sidebar component for the application.
 * It displays navigation links based on the user's role and includes a logout button.
 * On smaller screens, it's hidden, and navigation is handled by the AppHeader.
 */
export function AppSidebar() {
  const { user, logout } = useAuth();
  const pathname = usePathname();
  const router = useRouter();

  // Combine navigation items if the user is an administrator
  const navItems = user?.role === 'administrator' ? [...employeeNavItems, ...adminNavItems] : employeeNavItems;

  const handleLogout = () => {
    logout();
    router.push('/login');
  };

  return (
    <aside className="fixed inset-y-0 left-0 z-10 hidden w-14 flex-col border-r bg-background sm:flex">
      <TooltipProvider>
        <nav className="flex flex-col items-center gap-4 px-2 py-4">
          <Link
            href="/dashboard"
            className="group flex h-9 w-9 shrink-0 items-center justify-center gap-2 rounded-full bg-primary text-lg font-semibold text-primary-foreground md:h-8 md:w-8 md:text-base"
          >
            <LogoIcon className="h-4 w-4 transition-all group-hover:scale-110" />
            <span className="sr-only">TSI</span>
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
              <button
                onClick={handleLogout}
                className="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:text-foreground md:h-8 md:w-8"
              >
                <LogOut className="h-5 w-5" />
                <span className="sr-only">Logout</span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">Logout</TooltipContent>
          </Tooltip>
        </nav>
      </TooltipProvider>
    </aside>
  );
}
