'use client';

import {
  Bell,
  Home,
  LineChart,
  Package,
  Package2,
  PanelLeft,
  Search,
  ShoppingCart,
  BrainCircuit,
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet';
import Link from 'next/link';
import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { Avatar, AvatarFallback, AvatarImage } from './ui/avatar';
import { placeHolderImages } from '@/lib/placeholder-images';

export function AppHeader() {
  const { user, logout } = useAuth();
  const router = useRouter();

  const handleLogout = () => {
    logout();
    router.push('/login');
  };

  const userInitial = user?.firstName?.[0] ?? 'U';

  return (
    <header className="sticky top-0 z-30 flex h-14 items-center gap-4 border-b bg-background px-4 sm:static sm:h-auto sm:border-0 sm:bg-transparent sm:px-6">
      <Sheet>
        <SheetTrigger asChild>
          <Button size="icon" variant="outline" className="sm:hidden">
            <PanelLeft className="h-5 w-5" />
            <span className="sr-only">Toggle Menu</span>
          </Button>
        </SheetTrigger>
        <SheetContent side="left" className="sm:max-w-xs">
          <nav className="grid gap-6 text-lg font-medium">
            <Link
              href="#"
              className="group flex h-10 w-10 shrink-0 items-center justify-center gap-2 rounded-full bg-primary text-lg font-semibold text-primary-foreground md:text-base"
            >
              <Package2 className="h-5 w-5 transition-all group-hover:scale-110" />
              <span className="sr-only">TSI</span>
            </Link>
            <Link href="/dashboard" className="flex items-center gap-4 px-2.5 text-muted-foreground hover:text-foreground">
              <Home className="h-5 w-5" />
              Dashboard
            </Link>
            <Link href="/sessions" className="flex items-center gap-4 px-2.5 text-foreground">
              <ShoppingCart className="h-5 w-5" />
              Sessions
            </Link>
            {user?.role === 'administrator' && (
              <>
                <Link
                  href="/admin/ideas"
                  className="flex items-center gap-4 px-2.5 text-muted-foreground hover:text-foreground"
                >
                  <BrainCircuit className="h-5 w-5" />
                  Admin
                </Link>
              </>
            )}
          </nav>
        </SheetContent>
      </Sheet>

      <div className="relative ml-auto flex-1 md:grow-0">
        {/* Search can be implemented later */}
      </div>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="icon" className="overflow-hidden rounded-full">
            <Avatar>
              <AvatarImage src={placeHolderImages.find(img => img.id === "avatar")?.imageUrl} alt="User avatar" />
              <AvatarFallback>{userInitial}</AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuLabel>
            {user?.firstName} {user?.lastName}
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem>Settings</DropdownMenuItem>
          <DropdownMenuItem>Support</DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={handleLogout}>Logout</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </header>
  );
}
