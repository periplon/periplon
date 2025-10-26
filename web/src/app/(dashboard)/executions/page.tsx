'use client'

import { useQuery } from '@tanstack/react-query'
import Link from 'next/link'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatDate, formatDuration, getStatusColor } from '@/lib/utils'
import { Eye, XCircle } from 'lucide-react'

export default function ExecutionsPage() {
  const { data: executions, isLoading } = useQuery({
    queryKey: ['executions'],
    queryFn: () => apiClient.listExecutions(),
    refetchInterval: 5000, // Refresh every 5 seconds for real-time updates
  })

  const getExecutionDuration = (execution: any) => {
    if (!execution.started_at) return null
    const end = execution.completed_at
      ? new Date(execution.completed_at)
      : new Date()
    const start = new Date(execution.started_at)
    return end.getTime() - start.getTime()
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Executions</h1>
        <p className="text-muted-foreground">
          Monitor workflow execution status and history
        </p>
      </div>

      {isLoading ? (
        <div>Loading...</div>
      ) : executions && executions.length > 0 ? (
        <div className="space-y-3">
          {executions.map((execution) => {
            const duration = getExecutionDuration(execution)
            return (
              <Card key={execution.id} className="hover:shadow-md transition-shadow">
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex-1 space-y-2">
                      <div className="flex items-center gap-3">
                        <Badge className={getStatusColor(execution.status)}>
                          {execution.status}
                        </Badge>
                        <span className="font-medium text-sm">
                          {execution.workflow_id}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          v{execution.workflow_version}
                        </span>
                      </div>

                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span>{formatDate(execution.created_at)}</span>
                        {duration && (
                          <span>Duration: {formatDuration(duration)}</span>
                        )}
                        <span>Trigger: {execution.trigger_type}</span>
                        {execution.triggered_by && (
                          <span>By: {execution.triggered_by}</span>
                        )}
                      </div>

                      {execution.error && (
                        <div className="text-sm text-destructive">
                          Error: {execution.error}
                        </div>
                      )}
                    </div>

                    <div className="flex gap-2">
                      <Button variant="outline" size="sm" asChild>
                        <Link href={`/executions/${execution.id}`}>
                          <Eye className="h-3 w-3 mr-1" />
                          View
                        </Link>
                      </Button>
                      {execution.status === 'running' && (
                        <Button
                          variant="destructive"
                          size="sm"
                          onClick={() => apiClient.cancelExecution(execution.id)}
                        >
                          <XCircle className="h-3 w-3 mr-1" />
                          Cancel
                        </Button>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground">No executions yet</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
