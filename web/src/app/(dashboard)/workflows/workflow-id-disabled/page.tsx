'use client'

import { useQuery } from '@tanstack/react-query'
import { useParams } from 'next/navigation'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatDate } from '@/lib/utils'
import { Play, Edit, Trash2, ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { useState } from 'react'

export default function WorkflowDetailPage() {
  const params = useParams()
  const workflowId = params.id as string
  const [yamlView, setYamlView] = useState(true)

  const { data: workflow, isLoading } = useQuery({
    queryKey: ['workflow', workflowId],
    queryFn: () => apiClient.getWorkflow(workflowId),
    enabled: !!workflowId,
  })

  if (isLoading) {
    return <div>Loading...</div>
  }

  if (!workflow) {
    return <div>Workflow not found</div>
  }

  const yamlContent = `name: ${workflow.definition.name}
version: ${workflow.definition.version}
${workflow.definition.description ? `description: ${workflow.definition.description}\n` : ''}
agents:${Object.entries(workflow.definition.agents || {}).map(([key, agent]) => `
  ${key}:
    description: "${agent.description}"${agent.model ? `\n    model: ${agent.model}` : ''}${agent.tools ? `\n    tools: [${agent.tools.join(', ')}]` : ''}`).join('')}

tasks:${Object.entries(workflow.definition.tasks || {}).map(([key, task]) => `
  ${key}:
    description: "${task.description}"
    agent: ${task.agent}${task.depends_on ? `\n    depends_on: [${task.depends_on.join(', ')}]` : ''}${task.output ? `\n    output: ${task.output}` : ''}`).join('')}`

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" asChild>
          <Link href="/workflows">
            <ArrowLeft className="h-4 w-4" />
          </Link>
        </Button>
        <div className="flex-1">
          <h1 className="text-3xl font-bold">{workflow.name}</h1>
          <p className="text-muted-foreground">Version {workflow.version}</p>
        </div>
        <div className="flex gap-2">
          <Button variant="default" asChild>
            <Link href={`/workflows/${workflow.id}/execute`}>
              <Play className="h-4 w-4 mr-2" />
              Execute
            </Link>
          </Button>
          <Button variant="outline">
            <Edit className="h-4 w-4 mr-2" />
            Edit
          </Button>
          <Button variant="destructive">
            <Trash2 className="h-4 w-4 mr-2" />
            Delete
          </Button>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Status</CardTitle>
          </CardHeader>
          <CardContent>
            <Badge variant={workflow.is_active ? 'default' : 'secondary'}>
              {workflow.is_active ? 'Active' : 'Inactive'}
            </Badge>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Created</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm">{formatDate(workflow.created_at)}</p>
            {workflow.created_by && (
              <p className="text-xs text-muted-foreground">
                by {workflow.created_by}
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Updated</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm">{formatDate(workflow.updated_at)}</p>
          </CardContent>
        </Card>
      </div>

      {workflow.description && (
        <Card>
          <CardHeader>
            <CardTitle>Description</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">{workflow.description}</p>
          </CardContent>
        </Card>
      )}

      {workflow.tags && workflow.tags.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Tags</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {workflow.tags.map((tag) => (
                <Badge key={tag} variant="outline">
                  {tag}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Workflow Definition</CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setYamlView(!yamlView)}
            >
              {yamlView ? 'JSON' : 'YAML'}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <pre className="bg-muted p-4 rounded-lg overflow-x-auto">
            <code className="text-sm">
              {yamlView ? yamlContent : JSON.stringify(workflow.definition, null, 2)}
            </code>
          </pre>
        </CardContent>
      </Card>
    </div>
  )
}
