import React, { useState, useEffect } from 'react'
import { useICNClient } from '@icn/ts-sdk'

interface JobStatus {
  id: string
  submitter: string
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
  submitted_at: string
  started_at?: string
  completed_at?: string
  executor?: string
  cost: number
  progress?: number
  result?: {
    output: string
    exit_code: number
  }
  error?: string
}

interface JobSubmissionData {
  job_spec: {
    image: string
    command: string[]
    resources: {
      cpu_cores: number
      memory_mb: number
      storage_mb: number
    }
    environment?: Record<string, string>
  }
  submitter_did: string
  max_cost: number
  timeout_seconds?: number
}

interface JobMetrics {
  total_jobs: number
  pending_jobs: number
  running_jobs: number
  completed_jobs: number
  failed_jobs: number
  average_execution_time: number
}

function JobSubmissionModal({ isOpen, onClose, onSubmit, loading }: {
  isOpen: boolean
  onClose: () => void
  onSubmit: (data: JobSubmissionData) => Promise<void>
  loading: boolean
}) {
  const [activeTab, setActiveTab] = useState<'templates' | 'basic' | 'advanced'>('templates')
  const [formData, setFormData] = useState<JobSubmissionData>({
    job_spec: {
      image: '',
      command: [],
      resources: {
        cpu_cores: 1,
        memory_mb: 512,
        storage_mb: 1024
      }
    },
    submitter_did: 'did:key:example', // In real app, get from auth context
    max_cost: 100
  })
  const [commandText, setCommandText] = useState('')
  const [envText, setEnvText] = useState('')
  const [errors, setErrors] = useState<Record<string, string>>({})

  const jobTemplates = [
    {
      id: 'echo',
      name: 'Echo Test',
      description: 'Simple connectivity test',
      image: 'alpine:latest',
      command: ['echo', 'Hello ICN!'],
      cost: 10,
      timeout: 30
    },
    {
      id: 'python',
      name: 'Python Script',
      description: 'Execute Python code',
      image: 'python:3.9',
      command: ['python', '-c', 'print("Hello from Python!")'],
      cost: 50,
      timeout: 300
    },
    {
      id: 'compute',
      name: 'Heavy Compute',
      description: 'CPU-intensive task',
      image: 'alpine:latest',
      command: ['sh', '-c', 'for i in $(seq 1 1000000); do echo $i > /dev/null; done'],
      cost: 200,
      timeout: 120
    }
  ]

  const calculateEstimatedCost = () => {
    const baseCost = 10
    const cpuCost = formData.job_spec.resources.cpu_cores * 20
    const memoryCost = Math.ceil(formData.job_spec.resources.memory_mb / 128) * 15
    const storageCost = Math.ceil(formData.job_spec.resources.storage_mb / 100) * 5
    const timeCost = Math.ceil((formData.timeout_seconds || 300) / 60) * 2
    return baseCost + cpuCost + memoryCost + storageCost + timeCost
  }

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {}

    if (!formData.job_spec.image.trim()) {
      newErrors.image = 'Container image is required'
    }

    if (commandText.trim() === '') {
      newErrors.command = 'Command is required'
    }

    if (formData.max_cost < calculateEstimatedCost()) {
      newErrors.max_cost = `Budget too low. Estimated cost: ${calculateEstimatedCost()} mana`
    }

    if (envText.trim() && !isValidJSON(envText)) {
      newErrors.environment = 'Environment variables must be valid JSON'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const isValidJSON = (str: string): boolean => {
    try {
      JSON.parse(str)
      return true
    } catch {
      return false
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Parse command and environment
    const updatedFormData = {
      ...formData,
      job_spec: {
        ...formData.job_spec,
        command: commandText.split(' ').filter(c => c.trim()),
        environment: envText.trim() ? JSON.parse(envText) : undefined
      }
    }

    if (validateForm()) {
      await onSubmit(updatedFormData)
    }
  }

  const useTemplate = (template: typeof jobTemplates[0]) => {
    setFormData(prev => ({
      ...prev,
      job_spec: {
        ...prev.job_spec,
        image: template.image,
        command: template.command
      },
      max_cost: template.cost,
      timeout_seconds: template.timeout
    }))
    setCommandText(template.command.join(' '))
    setActiveTab('basic')
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg max-w-4xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-6 border-b border-gray-200">
          <div className="flex justify-between items-center">
            <h2 className="text-xl font-semibold text-gray-900">Submit New Job</h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              ✕
            </button>
          </div>
          
          {/* Tab Navigation */}
          <div className="flex space-x-4 mt-4">
            {[
              { id: 'templates', label: 'Templates' },
              { id: 'basic', label: 'Basic Config' },
              { id: 'advanced', label: 'Advanced' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`px-4 py-2 rounded-md text-sm font-medium ${
                  activeTab === tab.id
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        <form onSubmit={handleSubmit} className="p-6">
          {/* Templates Tab */}
          {activeTab === 'templates' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Choose a Template</h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {jobTemplates.map((template) => (
                  <div
                    key={template.id}
                    className="border border-gray-200 rounded-lg p-4 hover:border-blue-300 cursor-pointer"
                    onClick={() => useTemplate(template)}
                  >
                    <h4 className="font-medium text-gray-900">{template.name}</h4>
                    <p className="text-sm text-gray-600 mt-1">{template.description}</p>
                    <div className="mt-3 flex justify-between text-xs text-gray-500">
                      <span>{template.cost} mana</span>
                      <span>{template.timeout}s timeout</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Basic Configuration Tab */}
          {activeTab === 'basic' && (
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Container Image
                </label>
                <input
                  type="text"
                  value={formData.job_spec.image}
                  onChange={(e) => setFormData(prev => ({
                    ...prev,
                    job_spec: { ...prev.job_spec, image: e.target.value }
                  }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  placeholder="alpine:latest"
                />
                {errors.image && <p className="text-red-600 text-sm mt-1">{errors.image}</p>}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Command
                </label>
                <input
                  type="text"
                  value={commandText}
                  onChange={(e) => setCommandText(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  placeholder="echo Hello World"
                />
                {errors.command && <p className="text-red-600 text-sm mt-1">{errors.command}</p>}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    CPU Cores
                  </label>
                  <input
                    type="number"
                    min="1"
                    max="16"
                    value={formData.job_spec.resources.cpu_cores}
                    onChange={(e) => setFormData(prev => ({
                      ...prev,
                      job_spec: {
                        ...prev.job_spec,
                        resources: {
                          ...prev.job_spec.resources,
                          cpu_cores: parseInt(e.target.value) || 1
                        }
                      }
                    }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Memory (MB)
                  </label>
                  <input
                    type="number"
                    min="128"
                    max="8192"
                    step="128"
                    value={formData.job_spec.resources.memory_mb}
                    onChange={(e) => setFormData(prev => ({
                      ...prev,
                      job_spec: {
                        ...prev.job_spec,
                        resources: {
                          ...prev.job_spec.resources,
                          memory_mb: parseInt(e.target.value) || 512
                        }
                      }
                    }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Storage (MB)
                  </label>
                  <input
                    type="number"
                    min="100"
                    max="10240"
                    step="100"
                    value={formData.job_spec.resources.storage_mb}
                    onChange={(e) => setFormData(prev => ({
                      ...prev,
                      job_spec: {
                        ...prev.job_spec,
                        resources: {
                          ...prev.job_spec.resources,
                          storage_mb: parseInt(e.target.value) || 1024
                        }
                      }
                    }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Max Cost (Mana)
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={formData.max_cost}
                    onChange={(e) => setFormData(prev => ({
                      ...prev,
                      max_cost: parseInt(e.target.value) || 100
                    }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                  <p className="text-sm text-gray-500 mt-1">
                    Estimated: {calculateEstimatedCost()} mana
                  </p>
                  {errors.max_cost && <p className="text-red-600 text-sm mt-1">{errors.max_cost}</p>}
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Timeout (seconds)
                  </label>
                  <input
                    type="number"
                    min="10"
                    max="3600"
                    value={formData.timeout_seconds || 300}
                    onChange={(e) => setFormData(prev => ({
                      ...prev,
                      timeout_seconds: parseInt(e.target.value) || 300
                    }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Advanced Tab */}
          {activeTab === 'advanced' && (
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Environment Variables (JSON)
                </label>
                <textarea
                  value={envText}
                  onChange={(e) => setEnvText(e.target.value)}
                  rows={6}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 font-mono text-sm"
                  placeholder='{"NODE_ENV": "production", "DEBUG": "false"}'
                />
                {errors.environment && <p className="text-red-600 text-sm mt-1">{errors.environment}</p>}
              </div>

              <div className="bg-blue-50 p-4 rounded-md">
                <h4 className="font-medium text-blue-900 mb-2">🚀 Future Features</h4>
                <ul className="text-sm text-blue-700 space-y-1">
                  <li>• GPU resource allocation</li>
                  <li>• Network bandwidth limits</li>
                  <li>• Custom executor selection</li>
                  <li>• Job dependencies and scheduling</li>
                  <li>• Data input/output specifications</li>
                </ul>
              </div>
            </div>
          )}

          {/* Submit Button */}
          <div className="flex justify-end space-x-3 mt-8 pt-6 border-t border-gray-200">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            >
              {loading ? 'Submitting...' : 'Submit Job'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

function JobMetricsCards({ metrics, loading }: {
  metrics: JobMetrics | null
  loading: boolean
}) {
  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="bg-white rounded-lg border border-gray-200 p-4">
            <div className="animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-1/2 mb-2"></div>
              <div className="h-8 bg-gray-200 rounded w-3/4"></div>
            </div>
          </div>
        ))}
      </div>
    )
  }

  if (!metrics) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <p className="text-red-700">Failed to load job metrics</p>
      </div>
    )
  }

  const runningJobs = metrics.pending_jobs + metrics.running_jobs
  const successRate = metrics.total_jobs > 0 
    ? ((metrics.completed_jobs / metrics.total_jobs) * 100).toFixed(1)
    : '0'

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">Total Jobs</p>
            <p className="text-2xl font-bold text-blue-600">{metrics.total_jobs}</p>
          </div>
          <div className="text-2xl text-blue-500">📊</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">Running</p>
            <p className="text-2xl font-bold text-blue-600">{runningJobs}</p>
          </div>
          <div className="text-2xl text-blue-500">⚡</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">Completed</p>
            <p className="text-2xl font-bold text-green-600">{metrics.completed_jobs}</p>
          </div>
          <div className="text-2xl text-green-500">✅</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">Success Rate</p>
            <p className="text-2xl font-bold text-green-600">{successRate}%</p>
          </div>
          <div className="text-2xl text-green-500">🎯</div>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-gray-600">Avg Time</p>
            <p className="text-2xl font-bold text-purple-600">{metrics.average_execution_time}s</p>
          </div>
          <div className="text-2xl text-purple-500">⏱️</div>
        </div>
      </div>
    </div>
  )
}

// Custom hook for real-time job updates
function useRealtimeJobs() {
  const icnClient = useICNClient()
  const [jobs, setJobs] = useState<JobStatus[]>([])
  const [metrics, setMetrics] = useState<JobMetrics | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isConnected, setIsConnected] = useState(false)

  useEffect(() => {
    let interval: NodeJS.Timeout

    const fetchJobs = async () => {
      try {
        setError(null)
        const jobsList = await icnClient.mesh.listJobs()
        const metricsData = await icnClient.meshAdvanced.getMetrics()
        
        setJobs(jobsList)
        setMetrics(metricsData)
        setIsConnected(true)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch jobs')
        setIsConnected(false)
      } finally {
        setLoading(false)
      }
    }

    // Initial fetch
    fetchJobs()

    // Set up polling every 3 seconds for real-time updates
    interval = setInterval(fetchJobs, 3000)

    return () => {
      if (interval) clearInterval(interval)
    }
  }, [icnClient])

  return {
    jobs,
    metrics,
    loading,
    error,
    isConnected,
    refresh: () => setLoading(true)
  }
}

export function JobsPage() {
  const icnClient = useICNClient()
  const { jobs, metrics, loading, error, isConnected, refresh } = useRealtimeJobs()
  const [showSubmitModal, setShowSubmitModal] = useState(false)
  const [submitLoading, setSubmitLoading] = useState(false)
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [searchTerm, setSearchTerm] = useState('')

  const handleSubmitJob = async (jobData: JobSubmissionData) => {
    setSubmitLoading(true)
    try {
      await icnClient.mesh.submitJob(jobData)
      setShowSubmitModal(false)
      refresh() // Refresh the jobs list
    } catch (err) {
      console.error('Failed to submit job:', err)
    } finally {
      setSubmitLoading(false)
    }
  }

  const filteredJobs = jobs.filter(job => {
    const matchesStatus = statusFilter === 'all' || job.status.toLowerCase() === statusFilter
    const matchesSearch = !searchTerm || 
      job.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
      job.submitter.toLowerCase().includes(searchTerm.toLowerCase()) ||
      job.executor?.toLowerCase().includes(searchTerm.toLowerCase())
    
    return matchesStatus && matchesSearch
  })

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Completed': return 'bg-green-100 text-green-800'
      case 'Running': return 'bg-blue-100 text-blue-800'
      case 'Pending': return 'bg-yellow-100 text-yellow-800'
      case 'Failed': return 'bg-red-100 text-red-800'
      case 'Cancelled': return 'bg-gray-100 text-gray-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Mesh Jobs</h1>
          <p className="text-gray-600 mt-1">
            Manage and monitor distributed job execution
          </p>
        </div>
        <div className="flex items-center space-x-3">
          {/* Connection Status */}
          <div className="flex items-center space-x-2 text-sm">
            <div className={`w-2 h-2 rounded-full ${
              isConnected ? 'bg-green-500' : 'bg-red-500'
            }`}></div>
            <span className="text-gray-600">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
          
          <button
            onClick={() => setShowSubmitModal(true)}
            className="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 flex items-center space-x-2"
          >
            <span>⚡</span>
            <span>Submit Job</span>
          </button>
        </div>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p className="font-medium">Connection Error</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Job Metrics */}
      <JobMetricsCards metrics={metrics} loading={loading} />

      {/* Filters and Search */}
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-4 sm:space-y-0">
          <div className="flex items-center space-x-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Status Filter
              </label>
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
              >
                <option value="all">All Statuses</option>
                <option value="pending">Pending</option>
                <option value="running">Running</option>
                <option value="completed">Completed</option>
                <option value="failed">Failed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Search Jobs
            </label>
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search by ID, submitter, or executor..."
              className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 w-64"
            />
          </div>
        </div>
      </div>

      {/* Jobs List */}
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">
            Jobs ({filteredJobs.length})
          </h2>
        </div>

        {loading ? (
          <div className="p-8">
            <div className="space-y-4">
              {[1, 2, 3, 4, 5].map((i) => (
                <div key={i} className="animate-pulse flex space-x-4">
                  <div className="h-4 bg-gray-200 rounded w-20"></div>
                  <div className="h-4 bg-gray-200 rounded w-32"></div>
                  <div className="h-4 bg-gray-200 rounded w-24"></div>
                  <div className="h-4 bg-gray-200 rounded w-16"></div>
                </div>
              ))}
            </div>
          </div>
        ) : filteredJobs.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Job ID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Submitter
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Executor
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Cost
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Progress
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Submitted
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {filteredJobs.map((job) => (
                  <tr key={job.id} className="hover:bg-gray-50">
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-sm font-mono text-gray-900">
                        {job.id.substring(0, 8)}...
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(job.status)}`}>
                        {job.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-sm text-gray-900 font-mono">
                        {job.submitter.substring(0, 12)}...
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-sm text-gray-900 font-mono">
                        {job.executor ? `${job.executor.substring(0, 12)}...` : '-'}
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-sm text-gray-900">
                        {job.cost} mana
                      </div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      {job.progress !== undefined ? (
                        <div className="flex items-center">
                          <div className="w-16 bg-gray-200 rounded-full h-2 mr-2">
                            <div
                              className="bg-blue-600 h-2 rounded-full"
                              style={{ width: `${job.progress * 100}%` }}
                            ></div>
                          </div>
                          <span className="text-sm text-gray-600">
                            {Math.round(job.progress * 100)}%
                          </span>
                        </div>
                      ) : (
                        <span className="text-sm text-gray-400">-</span>
                      )}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-sm text-gray-900">
                        {new Date(job.submitted_at).toLocaleString()}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="p-8 text-center">
            <div className="text-4xl mb-4">⚡</div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No Jobs Found</h3>
            <p className="text-gray-600 mb-4">
              {searchTerm || statusFilter !== 'all' 
                ? 'No jobs match your current filters.'
                : 'Get started by submitting your first mesh job.'
              }
            </p>
            <button
              onClick={() => setShowSubmitModal(true)}
              className="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700"
            >
              Submit Your First Job
            </button>
          </div>
        )}
      </div>

      {/* Job Submission Modal */}
      <JobSubmissionModal
        isOpen={showSubmitModal}
        onClose={() => setShowSubmitModal(false)}
        onSubmit={handleSubmitJob}
        loading={submitLoading}
      />
    </div>
  )
} 