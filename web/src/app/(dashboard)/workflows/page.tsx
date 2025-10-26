'use client'

import { useQuery } from '@tanstack/react-query'
import Link from 'next/link'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { formatDate } from '@/lib/utils'
import { Plus, Play, Edit, Trash2 } from 'lucide-react'

export default function WorkflowsPage() {
  const { data: workflows, isLoading } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => apiClient.listWorkflows(),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Workflows</h1>
          <p className="text-muted-foreground">
            Manage and execute your workflow definitions
          </p>
        </div>
        <Button asChild>
          <Link href="/workflows/new">
            <Plus className="h-4 w-4 mr-2" />
            New Workflow
          </Link>
        </Button>
      </div>

      {isLoading ? (
        <div>Loading...</div>
      ) : workflows && workflows.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {workflows.map((workflow) => (
            <Card key={workflow.id} className="hover:shadow-lg transition-shadow">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <CardTitle className="text-lg">{workflow.name}</CardTitle>
                    <CardDescription>
                      Version {workflow.version}
                    </CardDescription>
                  </div>
                  <Badge variant={workflow.is_active ? 'default' : 'secondary'}>
                    {workflow.is_active ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                {workflow.description && (
                  <p className="text-sm text-muted-foreground line-clamp-2">
                    {workflow.description}
                  </p>
                )}

                {workflow.tags && workflow.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {workflow.tags.slice(0, 3).map((tag) => (
                      <Badge key={tag} variant="outline" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                    {workflow.tags.length > 3 && (
                      <Badge variant="outline" className="text-xs">
                        +{workflow.tags.length - 3}
                      </Badge>
                    )}
                  </div>
                )}

                <div className="text-xs text-muted-foreground">
                  Updated {formatDate(workflow.updated_at)}
                </div>

                <div className="flex gap-2">
                  <Button size="sm" variant="default" asChild className="flex-1">
                    <Link href={`/workflows/${workflow.id}/execute`}>
                      <Play className="h-3 w-3 mr-1" />
                      Execute
                    </Link>
                  </Button>
                  <Button size="sm" variant="outline" asChild>
                    <Link href={`/workflows/${workflow.id}`}>
                      <Edit className="h-3 w-3" />
                    </Link>
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground mb-4">No workflows yet</p>
            <Button asChild>
              <Link href="/workflows/new">
                <Plus className="h-4 w-4 mr-2" />
                Create your first workflow
              </Link>
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
