import { PageHeader } from '@/components/page-header';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { mockIdeas, mockUsers, mockSessions } from '@/lib/data';
import { Lightbulb, Users, Presentation, CheckCircle, Archive, Clock } from 'lucide-react';

export default function AdminDashboardPage() {
  const submittedIdeas = mockIdeas.filter((i) => i.status === 'submitted').length;
  const totalUsers = mockUsers.length;
  const activeSessions = mockSessions.filter((s) => s.status === 'active').length;

  return (
    <div className="container mx-auto p-0">
      <PageHeader
        title="Admin Dashboard"
        description="Oversee and manage all platform activity from here."
      />
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pending Ideas</CardTitle>
            <Lightbulb className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{submittedIdeas}</div>
            <p className="text-xs text-muted-foreground">ideas awaiting review or selection</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Users</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalUsers}</div>
            <p className="text-xs text-muted-foreground">employees and administrators</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Sessions</CardTitle>
            <Presentation className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{activeSessions}</div>
            <p className="text-xs text-muted-foreground">sessions currently in progress</p>
          </CardContent>
        </Card>
      </div>
      {/* More widgets and quick links can be added here */}
    </div>
  );
}
