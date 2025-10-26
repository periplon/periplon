'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatDate } from '@/lib/utils'
import { Plus, Play, Edit, Trash2, Clock } from 'lucide-react'
import Link from 'next/link'

export default function SchedulesPage() {
  const queryClient = useQueryClient()

  const { data: schedules, isLoading } = useQuery({
    queryKey: ['schedules'],
    queryFn: () => apiClient.listSchedules(),
  })

  const triggerMutation = useMutation({
    mutationFn: (id: string) => apiClient.triggerSchedule(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['schedules'] })
    },
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Schedules</h1>
          <p className="text-muted-foreground">
            Configure cron-based workflow execution schedules
          </p>
        </div>
        <Button asChild>
          <Link href="/schedules/new">
            <Plus className="h-4 w-4 mr-2" />
            New Schedule
          </Link>
        </Button>
      </div>

      {isLoading ? (
        <div>Loading...</div>
      ) : schedules && schedules.length > 0 ? (
        <div className="grid gap-4">
          {schedules.map((schedule) => (
            <Card key={schedule.id}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <Clock className="h-5 w-5 text-muted-foreground" />
                      <CardTitle className="text-lg">
                        {schedule.description || schedule.workflow_id}
                      </CardTitle>
                    </div>
                    <div className="flex items-center gap-3 text-sm text-muted-foreground">
                      <span className="font-mono bg-muted px-2 py-1 rounded">
                        {schedule.cron_expression}
                      </span>
                      <span>{schedule.timezone}</span>
                    </div>
                  </div>
                  <Badge variant={schedule.is_active ? 'default' : 'secondary'}>
                    {schedule.is_active ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Workflow:</span>
                    <Link
                      href={`/workflows/${schedule.workflow_id}`}
                      className="text-primary hover:underline"
                    >
                      {schedule.workflow_id}
                    </Link>
                  </div>
                  {schedule.last_run_at && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Last run:</span>
                      <span>{formatDate(schedule.last_run_at)}</span>
                    </div>
                  )}
                  {schedule.next_run_at && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Next run:</span>
                      <span className="font-medium">
                        {formatDate(schedule.next_run_at)}
                      </span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Created:</span>
                    <span>{formatDate(schedule.created_at)}</span>
                  </div>
                </div>

                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="default"
                    onClick={() => triggerMutation.mutate(schedule.id)}
                    disabled={triggerMutation.isPending}
                  >
                    <Play className="h-3 w-3 mr-1" />
                    Trigger Now
                  </Button>
                  <Button size="sm" variant="outline">
                    <Edit className="h-3 w-3 mr-1" />
                    Edit
                  </Button>
                  <Button size="sm" variant="destructive">
                    <Trash2 className="h-3 w-3 mr-1" />
                    Delete
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Clock className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground mb-4">No schedules yet</p>
            <Button asChild>
              <Link href="/schedules/new">
                <Plus className="h-4 w-4 mr-2" />
                Create your first schedule
              </Link>
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
