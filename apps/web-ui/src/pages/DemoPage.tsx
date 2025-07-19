import React from 'react'

export function DemoPage() {
  return (
    <div className="space-y-8">
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">
          ICN Federation Dashboard Demo
        </h1>
        <p className="text-xl text-gray-600 max-w-3xl mx-auto">
          This is a demonstration of the InterCooperative Network's Federation Onboarding & Governance UI. 
          The interface provides comprehensive tools for managing cooperative federations and participating 
          in democratic governance.
        </p>
      </div>

      {/* Features Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div className="card">
          <div className="card-body text-center">
            <div className="text-4xl mb-4">🏠</div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Dashboard</h3>
            <p className="text-gray-600 text-sm">
              Real-time monitoring of federation health, active cooperatives, governance proposals, 
              and network status with intuitive visualizations.
            </p>
          </div>
        </div>

        <div className="card">
          <div className="card-body text-center">
            <div className="text-4xl mb-4">🤝</div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Federation</h3>
            <p className="text-gray-600 text-sm">
              Create new federations or join existing ones. Manage trust relationships, 
              peer connections, and cooperative memberships.
            </p>
          </div>
        </div>

        <div className="card">
          <div className="card-body text-center">
            <div className="text-4xl mb-4">🗳️</div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Governance</h3>
            <p className="text-gray-600 text-sm">
              Participate in democratic decision-making with CCL-powered proposals, 
              template-based governance, and real-time voting.
            </p>
          </div>
        </div>
      </div>

      {/* Demo Features */}
      <div className="card">
        <div className="card-header">
          <h2 className="text-xl font-semibold text-gray-900">Demo Features</h2>
        </div>
        <div className="card-body space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div>
              <h3 className="font-semibold text-gray-900 mb-3">🚀 Federation Management</h3>
              <ul className="space-y-2 text-sm text-gray-600">
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Create new federations with DID generation and metadata
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Join existing federations via peer addresses
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Trust configuration and peer management
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Real-time federation status monitoring
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Health indicators for network, governance, and mesh
                </li>
              </ul>
            </div>

            <div>
              <h3 className="font-semibold text-gray-900 mb-3">🏛️ Governance System</h3>
              <ul className="space-y-2 text-sm text-gray-600">
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  CCL template-based proposal creation
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Interactive voting with real-time progress
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Quorum and threshold monitoring
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Multiple proposal types (membership, budget, governance)
                </li>
                <li className="flex items-start">
                  <span className="text-green-500 mr-2">✓</span>
                  Execution receipt viewing and history
                </li>
              </ul>
            </div>
          </div>

          <div>
            <h3 className="font-semibold text-gray-900 mb-3">🛠️ Technical Features</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div className="bg-blue-50 p-3 rounded-lg">
                <h4 className="font-medium text-blue-900 text-sm">TypeScript SDK</h4>
                <p className="text-blue-700 text-xs mt-1">
                  Comprehensive SDK with Federation, Governance, and utility APIs
                </p>
              </div>
              <div className="bg-green-50 p-3 rounded-lg">
                <h4 className="font-medium text-green-900 text-sm">React Context</h4>
                <p className="text-green-700 text-xs mt-1">
                  State management for federation and governance data
                </p>
              </div>
              <div className="bg-purple-50 p-3 rounded-lg">
                <h4 className="font-medium text-purple-900 text-sm">CCL Integration</h4>
                <p className="text-purple-700 text-xs mt-1">
                  Template system with parameter validation and code generation
                </p>
              </div>
              <div className="bg-yellow-50 p-3 rounded-lg">
                <h4 className="font-medium text-yellow-900 text-sm">Responsive UI</h4>
                <p className="text-yellow-700 text-xs mt-1">
                  Mobile-first design with Tailwind CSS and accessibility
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* CCL Templates Preview */}
      <div className="card">
        <div className="card-header">
          <h2 className="text-xl font-semibold text-gray-900">CCL Templates</h2>
          <p className="text-gray-600 text-sm mt-1">
            Cooperative Contract Language templates for common governance patterns
          </p>
        </div>
        <div className="card-body">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="border border-blue-200 rounded-lg p-4 bg-blue-50">
              <h3 className="font-semibold text-blue-900 mb-2">Member Admission</h3>
              <p className="text-blue-700 text-sm mb-3">
                Template for admitting new members with background checks and skill requirements
              </p>
              <div className="text-xs font-mono bg-blue-100 p-2 rounded">
                cooperative "{{name}}" {'{'}
                <br />
                &nbsp;&nbsp;propose admission {'{'}
                <br />
                &nbsp;&nbsp;&nbsp;&nbsp;candidate: "{{did}}"
                <br />
                &nbsp;&nbsp;{'}'}
                <br />
                {'}'}
              </div>
            </div>

            <div className="border border-green-200 rounded-lg p-4 bg-green-50">
              <h3 className="font-semibold text-green-900 mb-2">Budget Allocation</h3>
              <p className="text-green-700 text-sm mb-3">
                Template for budget allocation with accountability and reporting
              </p>
              <div className="text-xs font-mono bg-green-100 p-2 rounded">
                cooperative "{{name}}" {'{'}
                <br />
                &nbsp;&nbsp;propose budget {'{'}
                <br />
                &nbsp;&nbsp;&nbsp;&nbsp;amount: {{mana}}
                <br />
                &nbsp;&nbsp;{'}'}
                <br />
                {'}'}
              </div>
            </div>

            <div className="border border-purple-200 rounded-lg p-4 bg-purple-50">
              <h3 className="font-semibold text-purple-900 mb-2">Governance Change</h3>
              <p className="text-purple-700 text-sm mb-3">
                Template for changing governance rules with impact assessment
              </p>
              <div className="text-xs font-mono bg-purple-100 p-2 rounded">
                cooperative "{{name}}" {'{'}
                <br />
                &nbsp;&nbsp;propose governance {'{'}
                <br />
                &nbsp;&nbsp;&nbsp;&nbsp;rule: "{{rule}}"
                <br />
                &nbsp;&nbsp;{'}'}
                <br />
                {'}'}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Navigation Guide */}
      <div className="card">
        <div className="card-header">
          <h2 className="text-xl font-semibold text-gray-900">Navigation Guide</h2>
        </div>
        <div className="card-body">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div>
              <h3 className="font-semibold text-gray-900 mb-3">Getting Started</h3>
              <ol className="space-y-2 text-sm text-gray-600">
                <li className="flex">
                  <span className="bg-blue-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center mr-3 mt-0.5">1</span>
                  Visit the <strong>Dashboard</strong> to see federation overview and health metrics
                </li>
                <li className="flex">
                  <span className="bg-blue-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center mr-3 mt-0.5">2</span>
                  Explore <strong>Federation</strong> to create or join federations
                </li>
                <li className="flex">
                  <span className="bg-blue-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center mr-3 mt-0.5">3</span>
                  Use <strong>Governance</strong> to create proposals and vote on decisions
                </li>
                <li className="flex">
                  <span className="bg-blue-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center mr-3 mt-0.5">4</span>
                  Manage <strong>Cooperatives</strong> to add and organize federation members
                </li>
              </ol>
            </div>

            <div>
              <h3 className="font-semibold text-gray-900 mb-3">Demo Notes</h3>
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                <div className="flex">
                  <div className="text-yellow-600 mr-3">⚠️</div>
                  <div>
                    <p className="text-yellow-800 text-sm font-medium mb-2">Demo Environment</p>
                    <p className="text-yellow-700 text-xs">
                      This interface uses mock data and simulated API responses. 
                      In a production environment, all actions would interact with live ICN nodes, 
                      persist to the DAG, and propagate through the federation network.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Technical Architecture */}
      <div className="card">
        <div className="card-header">
          <h2 className="text-xl font-semibold text-gray-900">Technical Architecture</h2>
        </div>
        <div className="card-body">
          <div className="text-center mb-6">
            <div className="inline-flex items-center space-x-4 text-sm">
              <div className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full">React + TypeScript</div>
              <span className="text-gray-400">→</span>
              <div className="bg-green-100 text-green-800 px-3 py-1 rounded-full">ICN TypeScript SDK</div>
              <span className="text-gray-400">→</span>
              <div className="bg-purple-100 text-purple-800 px-3 py-1 rounded-full">ICN API Traits</div>
              <span className="text-gray-400">→</span>
              <div className="bg-orange-100 text-orange-800 px-3 py-1 rounded-full">ICN Core (Rust)</div>
            </div>
          </div>
          
          <div className="text-center text-gray-600 text-sm">
            <p>
              The UI is built as a modular React application that integrates with the ICN ecosystem 
              through a comprehensive TypeScript SDK. This enables type-safe interaction with 
              federation management, governance workflows, and cooperative coordination.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}