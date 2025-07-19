import React from 'react'
import { useFederation } from '../contexts/FederationContext'
import { useGovernance } from '../contexts/GovernanceContext'
import { FederationUtils, GovernanceUtils, ICNUtils } from '@icn/ts-sdk'

interface StatCardProps {
  title: string
  value: string | number
  subtitle?: string
  status?: 'success' | 'warning' | 'error' | 'info'
  loading?: boolean
}

function StatCard({ title, value, subtitle, status = 'info', loading }: StatCardProps) {
  const statusClasses = {
    success: 'bg-green-50 text-green-700 border-green-200',
    warning: 'bg-yellow-50 text-yellow-700 border-yellow-200',
    error: 'bg-red-50 text-red-700 border-red-200',
    info: 'bg-blue-50 text-blue-700 border-blue-200'
  }

  return (
    <div className={`rounded-lg border p-6 ${statusClasses[status]}`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium opacity-75">{title}</p>
          <p className="text-2xl font-bold">
            {loading ? (
              <div className="animate-pulse bg-current opacity-25 h-8 w-16 rounded"></div>
            ) : (
              value
            )}
          </p>
          {subtitle && (
            <p className="text-sm opacity-60 mt-1">{subtitle}</p>
          )}
        </div>
      </div>
    </div>
  )
}

interface HealthIndicatorProps {
  health: number
  label: string
}

function HealthIndicator({ health, label }: HealthIndicatorProps) {
  const getHealthColor = (score: number) => {
    if (score >= 80) return 'bg-green-500'
    if (score >= 60) return 'bg-yellow-500'
    return 'bg-red-500'
  }

  return (
    <div className="flex items-center space-x-3">
      <div className="flex-1">
        <div className="flex justify-between text-sm mb-1">
          <span>{label}</span>
          <span className="font-semibold">{health}%</span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-2">
          <div
            className={`h-2 rounded-full transition-all duration-300 ${getHealthColor(health)}`}
            style={{ width: `${health}%` }}
          />
        </div>
      </div>
    </div>
  )
}

export function Dashboard() {
  const {
    federationStatus,
    cooperatives,
    metadata,
    nodeInfo,
    nodeStatus,
    loading,
    error: federationError
  } = useFederation()

  const {
    proposals,
    activeProposals,
    loading: governanceLoading,
    error: governanceError
  } = useGovernance()

  const error = federationError || governanceError

  // Calculate health scores
  const overallHealth = metadata ? FederationUtils.calculateHealthScore(metadata) : 0
  const networkHealth = nodeStatus?.is_online ? 100 : 0
  const governanceHealth = metadata?.governance ? 
    Math.min((metadata.governance.activeProposals / Math.max(metadata.governance.totalProposals, 1)) * 100, 100) : 0

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Federation Dashboard</h1>
        <p className="text-gray-600 mt-2">
          Monitor and manage your cooperative federation
        </p>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p className="font-medium">Error</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Total Cooperatives"
          value={metadata?.totalCooperatives || 0}
          subtitle="Active in federation"
          status="success"
          loading={loading.cooperatives}
        />
        <StatCard
          title="Total Members"
          value={metadata?.totalMembers || 0}
          subtitle="Across all cooperatives"
          status="info"
          loading={loading.cooperatives}
        />
        <StatCard
          title="Active Proposals"
          value={activeProposals.length}
          subtitle={`${proposals.length} total proposals`}
          status="warning"
          loading={governanceLoading.proposals}
        />
        <StatCard
          title="Network Peers"
          value={federationStatus?.peer_count || 0}
          subtitle="Connected peers"
          status={federationStatus?.peer_count ? 'success' : 'error'}
          loading={loading.federationStatus}
        />
      </div>

      {/* Health Indicators */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Federation Health</h2>
        <div className="space-y-4">
          <HealthIndicator health={overallHealth} label="Overall Health" />
          <HealthIndicator health={networkHealth} label="Network Connectivity" />
          <HealthIndicator health={governanceHealth} label="Governance Activity" />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Recent Cooperatives */}
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Active Cooperatives</h2>
          {loading.cooperatives ? (
            <div className="space-y-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="animate-pulse">
                  <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              ))}
            </div>
          ) : cooperatives.length > 0 ? (
            <div className="space-y-4">
              {cooperatives.slice(0, 5).map((coop) => (
                <div key={coop.did} className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-gray-900">{coop.name}</p>
                    <p className="text-sm text-gray-600">
                      {coop.memberCount} members • {coop.reputation}% reputation
                    </p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        coop.status === 'active'
                          ? 'bg-green-100 text-green-800'
                          : 'bg-gray-100 text-gray-800'
                      }`}
                    >
                      {coop.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500 text-center py-8">No cooperatives found</p>
          )}
        </div>

        {/* Recent Proposals */}
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Recent Proposals</h2>
          {governanceLoading.proposals ? (
            <div className="space-y-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="animate-pulse">
                  <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              ))}
            </div>
          ) : proposals.length > 0 ? (
            <div className="space-y-4">
              {proposals.slice(0, 5).map((proposal) => (
                <div key={proposal.id} className="border-l-4 border-blue-400 pl-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-gray-900">
                        {GovernanceUtils.formatProposalType(proposal.proposal_type)}
                      </p>
                      <p className="text-sm text-gray-600">
                        {GovernanceUtils.generateProposalSummary(proposal.proposal_type)}
                      </p>
                      <p className="text-xs text-gray-500 mt-1">
                        {proposal.voting_deadline && ICNUtils.getTimeRemaining(proposal.voting_deadline)}
                      </p>
                    </div>
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        proposal.status === 'Open'
                          ? 'bg-blue-100 text-blue-800'
                          : proposal.status === 'Closed'
                          ? 'bg-gray-100 text-gray-800'
                          : 'bg-green-100 text-green-800'
                      }`}
                    >
                      {proposal.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500 text-center py-8">No proposals found</p>
          )}
        </div>
      </div>

      {/* System Information */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">System Information</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <h3 className="font-medium text-gray-900 mb-2">Node Information</h3>
            {loading.nodeInfo ? (
              <div className="animate-pulse space-y-2">
                <div className="h-3 bg-gray-200 rounded w-full"></div>
                <div className="h-3 bg-gray-200 rounded w-3/4"></div>
              </div>
            ) : nodeInfo ? (
              <div className="text-sm text-gray-600 space-y-1">
                <p><span className="font-medium">Name:</span> {nodeInfo.name}</p>
                <p><span className="font-medium">Version:</span> {nodeInfo.version}</p>
                <p><span className="font-medium">DID:</span> {nodeInfo.did}</p>
              </div>
            ) : (
              <p className="text-gray-500 text-sm">No node information available</p>
            )}
          </div>
          
          <div>
            <h3 className="font-medium text-gray-900 mb-2">Network Status</h3>
            {nodeStatus ? (
              <div className="text-sm text-gray-600 space-y-1">
                <p>
                  <span className="font-medium">Status:</span>{' '}
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                      nodeStatus.is_online
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {nodeStatus.is_online ? 'Online' : 'Offline'}
                  </span>
                </p>
                <p><span className="font-medium">Peers:</span> {nodeStatus.peer_count}</p>
                <p><span className="font-medium">Block Height:</span> {nodeStatus.current_block_height}</p>
              </div>
            ) : (
              <p className="text-gray-500 text-sm">No network status available</p>
            )}
          </div>

          <div>
            <h3 className="font-medium text-gray-900 mb-2">DAG Status</h3>
            {metadata?.dag ? (
              <div className="text-sm text-gray-600 space-y-1">
                <p><span className="font-medium">Blocks:</span> {metadata.dag.blockCount.toLocaleString()}</p>
                <p>
                  <span className="font-medium">Sync:</span>{' '}
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                      metadata.dag.syncStatus === 'synced'
                        ? 'bg-green-100 text-green-800'
                        : metadata.dag.syncStatus === 'syncing'
                        ? 'bg-yellow-100 text-yellow-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {metadata.dag.syncStatus}
                  </span>
                </p>
              </div>
            ) : (
              <p className="text-gray-500 text-sm">No DAG status available</p>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}