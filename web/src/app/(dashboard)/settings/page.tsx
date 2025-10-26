'use client'

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api-client'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { formatDate } from '@/lib/utils'
import { Plus, Copy, RotateCw, Trash2, Eye, EyeOff } from 'lucide-react'

export default function SettingsPage() {
  const queryClient = useQueryClient()
  const [showNewKey, setShowNewKey] = useState(false)
  const [newKeyData, setNewKeyData] = useState<any>(null)
  const [newKeyForm, setNewKeyForm] = useState({
    name: '',
    description: '',
    scopes: ['workflows:read', 'workflows:write', 'executions:read'],
    expires_in_days: 90,
  })

  const { data: apiKeys, isLoading } = useQuery({
    queryKey: ['api-keys'],
    queryFn: () => apiClient.listApiKeys(),
  })

  const createKeyMutation = useMutation({
    mutationFn: (data: any) => apiClient.createApiKey(data),
    onSuccess: (data) => {
      setNewKeyData(data)
      setShowNewKey(false)
      queryClient.invalidateQueries({ queryKey: ['api-keys'] })
    },
  })

  const revokeKeyMutation = useMutation({
    mutationFn: (id: string) => apiClient.revokeApiKey(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['api-keys'] })
    },
  })

  const rotateKeyMutation = useMutation({
    mutationFn: (id: string) => apiClient.rotateApiKey(id),
    onSuccess: (data) => {
      setNewKeyData(data)
      queryClient.invalidateQueries({ queryKey: ['api-keys'] })
    },
  })

  const handleCreateKey = (e: React.FormEvent) => {
    e.preventDefault()
    createKeyMutation.mutate(newKeyForm)
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-muted-foreground">
          Manage your account settings and API keys
        </p>
      </div>

      {/* New Key Display */}
      {newKeyData && (
        <Card className="border-green-500">
          <CardHeader>
            <CardTitle className="text-green-600">New API Key Created</CardTitle>
            <CardDescription>
              Save this key now! You won&apos;t be able to see it again.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-muted p-3 rounded text-sm break-all">
                {newKeyData.key}
              </code>
              <Button
                size="icon"
                variant="outline"
                onClick={() => copyToClipboard(newKeyData.key)}
              >
                <Copy className="h-4 w-4" />
              </Button>
            </div>
            <Button onClick={() => setNewKeyData(null)} variant="outline">
              I&apos;ve saved my key
            </Button>
          </CardContent>
        </Card>
      )}

      {/* API Keys */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>API Keys</CardTitle>
              <CardDescription>
                Manage API keys for programmatic access
              </CardDescription>
            </div>
            <Button onClick={() => setShowNewKey(!showNewKey)}>
              <Plus className="h-4 w-4 mr-2" />
              New API Key
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {showNewKey && (
            <form onSubmit={handleCreateKey} className="space-y-4 p-4 border rounded-lg">
              <div className="space-y-2">
                <label className="text-sm font-medium">Name</label>
                <Input
                  placeholder="My API Key"
                  value={newKeyForm.name}
                  onChange={(e) =>
                    setNewKeyForm({ ...newKeyForm, name: e.target.value })
                  }
                  required
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Description</label>
                <Input
                  placeholder="Used for production deployments"
                  value={newKeyForm.description}
                  onChange={(e) =>
                    setNewKeyForm({ ...newKeyForm, description: e.target.value })
                  }
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">
                  Expires in (days)
                </label>
                <Input
                  type="number"
                  min="1"
                  max="365"
                  value={newKeyForm.expires_in_days}
                  onChange={(e) =>
                    setNewKeyForm({
                      ...newKeyForm,
                      expires_in_days: parseInt(e.target.value),
                    })
                  }
                  required
                />
              </div>
              <div className="flex gap-2">
                <Button
                  type="submit"
                  disabled={createKeyMutation.isPending}
                >
                  Create Key
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setShowNewKey(false)}
                >
                  Cancel
                </Button>
              </div>
            </form>
          )}

          {isLoading ? (
            <div>Loading...</div>
          ) : apiKeys && apiKeys.length > 0 ? (
            <div className="space-y-3">
              {apiKeys.map((key) => (
                <div
                  key={key.id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{key.name || 'Unnamed Key'}</span>
                      <Badge variant={key.is_active ? 'default' : 'secondary'}>
                        {key.is_active ? 'Active' : 'Revoked'}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <code className="text-xs">{key.key_prefix}...</code>
                      <span>Created {formatDate(key.created_at)}</span>
                      {key.expires_at && (
                        <span>Expires {formatDate(key.expires_at)}</span>
                      )}
                      {key.last_used_at && (
                        <span>Last used {formatDate(key.last_used_at)}</span>
                      )}
                    </div>
                    {key.description && (
                      <p className="text-sm text-muted-foreground">
                        {key.description}
                      </p>
                    )}
                    {key.scopes && key.scopes.length > 0 && (
                      <div className="flex flex-wrap gap-1 mt-2">
                        {key.scopes.map((scope) => (
                          <Badge key={scope} variant="outline" className="text-xs">
                            {scope}
                          </Badge>
                        ))}
                      </div>
                    )}
                  </div>
                  <div className="flex gap-2">
                    {key.is_active && (
                      <>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => rotateKeyMutation.mutate(key.id)}
                          disabled={rotateKeyMutation.isPending}
                        >
                          <RotateCw className="h-3 w-3 mr-1" />
                          Rotate
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onClick={() => revokeKeyMutation.mutate(key.id)}
                          disabled={revokeKeyMutation.isPending}
                        >
                          <Trash2 className="h-3 w-3 mr-1" />
                          Revoke
                        </Button>
                      </>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-center text-muted-foreground py-8">
              No API keys yet
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
