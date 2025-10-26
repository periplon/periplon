'use client'

import { useQuery } from '@tanstack/react-query'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { formatDate, getStatusColor } from '@/lib/utils'
import { Activity, CheckCircle2, Clock, TrendingUp } from 'lucide-react'
import Link from 'next/link'

export default function DashboardPage() {
  const { data: metrics, isLoading } = useQuery({
    queryKey: ['dashboard-metrics'],
    queryFn: () => apiClient.getDashboardMetrics(),
    refetchInterval: 30000, // Refresh every 30 seconds
  })

  if (isLoading) {
    return <div>Loading...</div>
  }

  const stats = [
    {
      title: 'Total Workflows',
      value: metrics?.total_workflows || 0,
      icon: Activity,
      color: 'text-blue-600',
    },
    {
      title: 'Active Executions',
      value: metrics?.active_executions || 0,
      icon: Clock,
      color: 'text-orange-600',
    },
    {
      title: 'Total Executions',
      value: metrics?.total_executions || 0,
      icon: TrendingUp,
      color: 'text-purple-600',
    },
    {
      title: 'Success Rate',
      value: `${((metrics?.success_rate || 0) * 100).toFixed(1)}%`,
      icon: CheckCircle2,
      color: 'text-green-600',
    },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Welcome back! Here&apos;s an overview of your workflows.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.title}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">
                {stat.title}
              </CardTitle>
              <stat.icon className={`h-4 w-4 ${stat.color}`} />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stat.value}</div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Recent Executions */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Executions</CardTitle>
        </CardHeader>
        <CardContent>
          {metrics?.recent_executions && metrics.recent_executions.length > 0 ? (
            <div className="space-y-4">
              {metrics.recent_executions.map((execution) => (
                <Link
                  key={execution.id}
                  href={`/executions/${execution.id}`}
                  className="flex items-center justify-between p-4 rounded-lg border hover:bg-accent transition-colors"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <Badge className={getStatusColor(execution.status)}>
                        {execution.status}
                      </Badge>
                      <span className="font-medium">
                        {execution.workflow_id}
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {formatDate(execution.created_at)}
                    </p>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {execution.trigger_type}
                  </div>
                </Link>
              ))}
            </div>
          ) : (
            <p className="text-center text-muted-foreground py-8">
              No recent executions
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
