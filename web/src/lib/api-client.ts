import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'
import type {
  Workflow,
  Execution,
  Schedule,
  Organization,
  Team,
  ApiKey,
  User,
  LoginRequest,
  LoginResponse,
  ExecutionLog,
  DashboardMetrics,
} from '@/types'

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

class ApiClient {
  private client: AxiosInstance

  constructor() {
    this.client = axios.create({
      baseURL: `${API_URL}/api/v1`,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    // Request interceptor to add auth token
    this.client.interceptors.request.use((config) => {
      const token = typeof window !== 'undefined' ? localStorage.getItem('access_token') : null
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }
      return config
    })

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      async (error) => {
        if (error.response?.status === 401) {
          // Try to refresh token
          const refreshToken = typeof window !== 'undefined' ? localStorage.getItem('refresh_token') : null
          if (refreshToken) {
            try {
              const response = await this.refreshToken(refreshToken)
              if (typeof window !== 'undefined') {
                localStorage.setItem('access_token', response.access_token)
                localStorage.setItem('refresh_token', response.refresh_token)
              }
              // Retry original request
              error.config.headers.Authorization = `Bearer ${response.access_token}`
              return this.client.request(error.config)
            } catch (refreshError) {
              // Refresh failed, logout
              if (typeof window !== 'undefined') {
                localStorage.removeItem('access_token')
                localStorage.removeItem('refresh_token')
                window.location.href = '/login'
              }
            }
          }
        }
        return Promise.reject(error)
      }
    )
  }

  // Authentication
  async login(data: LoginRequest): Promise<LoginResponse> {
    const response = await this.client.post<LoginResponse>('/auth/login', data)
    return response.data
  }

  async logout(): Promise<void> {
    await this.client.post('/auth/logout')
  }

  async refreshToken(refreshToken: string): Promise<LoginResponse> {
    const response = await this.client.post<LoginResponse>('/auth/refresh', {
      refresh_token: refreshToken,
    })
    return response.data
  }

  async getCurrentUser(): Promise<User> {
    const response = await this.client.get<User>('/auth/me')
    return response.data
  }

  // Workflows
  async listWorkflows(params?: any): Promise<Workflow[]> {
    const response = await this.client.get<Workflow[]>('/workflows', { params })
    return response.data
  }

  async getWorkflow(id: string): Promise<Workflow> {
    const response = await this.client.get<Workflow>(`/workflows/${id}`)
    return response.data
  }

  async createWorkflow(data: any): Promise<{ id: string }> {
    const response = await this.client.post<{ id: string }>('/workflows', data)
    return response.data
  }

  async updateWorkflow(id: string, data: any): Promise<void> {
    await this.client.put(`/workflows/${id}`, data)
  }

  async deleteWorkflow(id: string): Promise<void> {
    await this.client.delete(`/workflows/${id}`)
  }

  async validateWorkflow(definition: any): Promise<{ valid: boolean; errors?: string[] }> {
    const response = await this.client.post('/workflows/validate', { definition })
    return response.data
  }

  // Executions
  async listExecutions(params?: any): Promise<Execution[]> {
    const response = await this.client.get<Execution[]>('/executions', { params })
    return response.data
  }

  async getExecution(id: string): Promise<Execution> {
    const response = await this.client.get<Execution>(`/executions/${id}`)
    return response.data
  }

  async createExecution(data: any): Promise<{ id: string }> {
    const response = await this.client.post<{ id: string }>('/executions', data)
    return response.data
  }

  async cancelExecution(id: string): Promise<void> {
    await this.client.post(`/executions/${id}/cancel`)
  }

  async getExecutionLogs(id: string, limit?: number): Promise<ExecutionLog[]> {
    const response = await this.client.get<ExecutionLog[]>(`/executions/${id}/logs`, {
      params: { limit },
    })
    return response.data
  }

  // Schedules
  async listSchedules(params?: any): Promise<Schedule[]> {
    const response = await this.client.get<Schedule[]>('/schedules', { params })
    return response.data
  }

  async getSchedule(id: string): Promise<Schedule> {
    const response = await this.client.get<Schedule>(`/schedules/${id}`)
    return response.data
  }

  async createSchedule(data: any): Promise<{ id: string }> {
    const response = await this.client.post<{ id: string }>('/schedules', data)
    return response.data
  }

  async updateSchedule(id: string, data: any): Promise<void> {
    await this.client.put(`/schedules/${id}`, data)
  }

  async deleteSchedule(id: string): Promise<void> {
    await this.client.delete(`/schedules/${id}`)
  }

  async triggerSchedule(id: string): Promise<void> {
    await this.client.post(`/schedules/${id}/trigger`)
  }

  // Organizations
  async listOrganizations(params?: any): Promise<Organization[]> {
    const response = await this.client.get<Organization[]>('/organizations', { params })
    return response.data
  }

  async getOrganization(id: string): Promise<Organization> {
    const response = await this.client.get<Organization>(`/organizations/${id}`)
    return response.data
  }

  async createOrganization(data: any): Promise<{ id: string }> {
    const response = await this.client.post<{ id: string }>('/organizations', data)
    return response.data
  }

  async updateOrganization(id: string, data: any): Promise<void> {
    await this.client.put(`/organizations/${id}`, data)
  }

  async deleteOrganization(id: string): Promise<void> {
    await this.client.delete(`/organizations/${id}`)
  }

  // Teams
  async listTeams(params?: any): Promise<Team[]> {
    const response = await this.client.get<Team[]>('/teams', { params })
    return response.data
  }

  async getTeam(id: string): Promise<Team> {
    const response = await this.client.get<Team>(`/teams/${id}`)
    return response.data
  }

  async createTeam(data: any): Promise<{ id: string }> {
    const response = await this.client.post<{ id: string }>('/teams', data)
    return response.data
  }

  async updateTeam(id: string, data: any): Promise<void> {
    await this.client.put(`/teams/${id}`, data)
  }

  async deleteTeam(id: string): Promise<void> {
    await this.client.delete(`/teams/${id}`)
  }

  // API Keys
  async listApiKeys(): Promise<ApiKey[]> {
    const response = await this.client.get<ApiKey[]>('/api-keys')
    return response.data
  }

  async getApiKey(id: string): Promise<ApiKey> {
    const response = await this.client.get<ApiKey>(`/api-keys/${id}`)
    return response.data
  }

  async createApiKey(data: any): Promise<any> {
    const response = await this.client.post('/api-keys', data)
    return response.data
  }

  async updateApiKey(id: string, data: any): Promise<void> {
    await this.client.put(`/api-keys/${id}`, data)
  }

  async revokeApiKey(id: string): Promise<void> {
    await this.client.delete(`/api-keys/${id}`)
  }

  async rotateApiKey(id: string): Promise<any> {
    const response = await this.client.post(`/api-keys/${id}/rotate`)
    return response.data
  }

  // Dashboard
  async getDashboardMetrics(): Promise<DashboardMetrics> {
    const response = await this.client.get<DashboardMetrics>('/metrics/overview')
    return response.data
  }

  // Health
  async getHealth(): Promise<any> {
    const response = await this.client.get('/health')
    return response.data
  }
}

export const apiClient = new ApiClient()
export default apiClient
