/**
 * E2E Integration Tests for ICN Frontend-Backend Flows
 * 
 * This test suite validates actual frontend to backend API integration:
 * - Node connection and health checks
 * - Job submission and management flows
 * - Identity/DID operations 
 * - Governance proposals and voting
 * - Federation management
 * 
 * Tests can run against:
 * - Local development node (http://localhost:8080)
 * - Docker devnet federation
 * - Mock server for offline testing
 */

import { ICNClient } from './client';
import { createStorage } from './storage';

// Test configuration
const TEST_CONFIG = {
  nodeEndpoint: process.env.ICN_TEST_ENDPOINT || 'http://localhost:8080',
  network: (process.env.ICN_TEST_NETWORK || 'devnet') as 'mainnet' | 'testnet' | 'devnet',
  timeout: 30000,
  useOfflineMode: process.env.ICN_OFFLINE_MODE === 'true',
};

// Mock server responses for offline testing
const MOCK_RESPONSES = {
  systemInfo: {
    version: '0.2.0',
    name: 'ICN Node (Mock)',
    status_message: 'Node is operational',
  },
  nodeStatus: {
    is_online: true,
    peer_count: 3,
    current_block_height: 100,
    version: '0.2.0',
  },
  jobSubmission: {
    job_id: 'job_12345',
  },
  jobStatus: {
    job_id: 'job_12345',
    status: 'Completed',
    result: { output: 'Hello, World!' },
    submitter_did: 'did:key:test',
    created_at: new Date().toISOString(),
  },
  proposalSubmission: {
    proposal_id: 'prop_67890',
  },
  federationStatus: {
    name: 'Test Federation',
    total_members: 5,
    status: 'active',
  },
};

class MockFetch {
  async fetch(url: string, options?: RequestInit): Promise<Response> {
    console.log(`[MOCK] ${options?.method || 'GET'} ${url}`);
    
    // Parse the endpoint and return mock responses
    if (url.includes('/system/info')) {
      return this.mockResponse(MOCK_RESPONSES.systemInfo);
    }
    if (url.includes('/system/status')) {
      return this.mockResponse(MOCK_RESPONSES.nodeStatus);
    }
    if (url.includes('/mesh/jobs') && options?.method === 'POST') {
      return this.mockResponse(MOCK_RESPONSES.jobSubmission);
    }
    if (url.includes('/mesh/jobs/')) {
      return this.mockResponse(MOCK_RESPONSES.jobStatus);
    }
    if (url.includes('/governance/proposals') && options?.method === 'POST') {
      return this.mockResponse(MOCK_RESPONSES.proposalSubmission);
    }
    if (url.includes('/federation/status')) {
      return this.mockResponse(MOCK_RESPONSES.federationStatus);
    }
    
    // Default mock response
    return this.mockResponse({ error: 'Not implemented in mock' }, 501);
  }
  
  private mockResponse(data: any, status = 200): Response {
    return new Response(JSON.stringify(data), {
      status,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

// Test runner for integration tests
class IntegrationTestRunner {
  private client: ICNClient;
  private tests: Array<{
    name: string;
    category: string;
    fn: () => Promise<void>;
    skip?: boolean;
  }> = [];
  
  private results = {
    passed: 0,
    failed: 0,
    skipped: 0,
    failures: [] as string[],
  };

  constructor(useOfflineMode = false) {
    // Setup mock fetch for offline mode
    if (useOfflineMode) {
      const mockFetch = new MockFetch();
      global.fetch = mockFetch.fetch.bind(mockFetch);
    }

    this.client = new ICNClient({
      nodeEndpoint: TEST_CONFIG.nodeEndpoint,
      network: TEST_CONFIG.network,
      timeout: TEST_CONFIG.timeout,
      storage: createStorage('@icn-test:'),
    });
  }

  test(category: string, name: string, fn: () => Promise<void>, skip = false) {
    this.tests.push({ name, category, fn, skip });
  }

  async run() {
    console.log('🚀 Running ICN Frontend-Backend Integration Tests');
    console.log(`📍 Endpoint: ${TEST_CONFIG.nodeEndpoint}`);
    console.log(`🌐 Network: ${TEST_CONFIG.network}`);
    console.log(`📴 Offline Mode: ${TEST_CONFIG.useOfflineMode ? 'Yes' : 'No'}\n`);

    const categories = [...new Set(this.tests.map(t => t.category))];
    
    for (const category of categories) {
      console.log(`\n📂 ${category} Tests:`);
      const categoryTests = this.tests.filter(t => t.category === category);
      
      for (const test of categoryTests) {
        if (test.skip) {
          console.log(`⏭️  ${test.name} (skipped)`);
          this.results.skipped++;
          continue;
        }

        try {
          console.log(`⏳ ${test.name}...`);
          await test.fn();
          console.log(`✅ ${test.name}`);
          this.results.passed++;
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error);
          console.log(`❌ ${test.name}: ${errorMsg}`);
          this.results.failed++;
          this.results.failures.push(`${test.name}: ${errorMsg}`);
        }
      }
    }

    this.printSummary();
    return this.results;
  }

  private printSummary() {
    const total = this.results.passed + this.results.failed + this.results.skipped;
    console.log(`\n📊 Integration Test Results:`);
    console.log(`   ✅ Passed: ${this.results.passed}`);
    console.log(`   ❌ Failed: ${this.results.failed}`);
    console.log(`   ⏭️  Skipped: ${this.results.skipped}`);
    console.log(`   📈 Success Rate: ${total > 0 ? ((this.results.passed / (this.results.passed + this.results.failed)) * 100).toFixed(1) : 0}%`);

    if (this.results.failures.length > 0) {
      console.log('\n❌ Failures:');
      this.results.failures.forEach((failure, index) => {
        console.log(`   ${index + 1}. ${failure}`);
      });
    }
  }

  // Helper assertion functions
  async assert(condition: boolean, message: string) {
    if (!condition) {
      throw new Error(message);
    }
  }

  async assertEqual<T>(actual: T, expected: T, message?: string) {
    if (actual !== expected) {
      throw new Error(message || `Expected ${expected}, got ${actual}`);
    }
  }

  async assertResponseOk(response: any, operation: string) {
    if (!response) {
      throw new Error(`${operation} returned null/undefined response`);
    }
  }
}

// Define the integration tests
export async function runIntegrationTests(useOfflineMode = false) {
  const runner = new IntegrationTestRunner(useOfflineMode);

  // 1. System and Connection Tests
  runner.test('System', 'Connection and Basic Health', async () => {
    try {
      await runner.client.connect();
      const connectionState = runner.client.getConnectionState();
      await runner.assert(connectionState.connected, 'Should be connected to node');
    } catch (error) {
      if (!useOfflineMode) {
        throw error; // Re-throw in online mode
      }
      // In offline mode, expect connection to fail but that's OK
      console.log('  📴 Connection failed as expected in offline mode');
    }
  });

  runner.test('System', 'Node Information Retrieval', async () => {
    const nodeInfo = await runner.client.system.getInfo();
    await runner.assertResponseOk(nodeInfo, 'getInfo');
    await runner.assert(nodeInfo.version !== undefined, 'Node should return version info');
    await runner.assert(nodeInfo.name !== undefined, 'Node should return name');
  });

  runner.test('System', 'Node Status Check', async () => {
    const nodeStatus = await runner.client.system.getStatus();
    await runner.assertResponseOk(nodeStatus, 'getStatus');
    await runner.assert(typeof nodeStatus.is_online === 'boolean', 'Should return online status');
    await runner.assert(typeof nodeStatus.peer_count === 'number', 'Should return peer count');
  });

  // 2. Job Management Tests  
  runner.test('Mesh Computing', 'Job Submission Flow', async () => {
    const jobSpec = {
      name: 'test-job',
      image: 'hello-world',
      command: ['echo', 'Hello from ICN!'],
      resources: {
        cpu: '100m',
        memory: '128Mi',
      },
    };

    const jobId = await runner.client.submitJob(jobSpec, {
      maxCost: 100,
      priority: 'normal',
    });
    
    await runner.assertResponseOk(jobId, 'submitJob');
    await runner.assert(typeof jobId === 'string', 'Job ID should be a string');
    await runner.assert(jobId.length > 0, 'Job ID should not be empty');
  });

  runner.test('Mesh Computing', 'Job Status Retrieval', async () => {
    // First submit a job to get an ID
    const jobSpec = {
      name: 'status-test-job',
      image: 'hello-world',
      command: ['echo', 'Status test'],
    };
    
    const jobId = await runner.client.submitJob(jobSpec);
    const jobStatus = await runner.client.getJob(jobId);
    
    await runner.assertResponseOk(jobStatus, 'getJob');
    await runner.assert(jobStatus.job_id === jobId, 'Job status should return correct job ID');
    await runner.assert(jobStatus.status !== undefined, 'Job should have status');
  });

  runner.test('Mesh Computing', 'Job List Retrieval', async () => {
    const jobs = await runner.client.listJobs();
    await runner.assertResponseOk(jobs, 'listJobs');
    await runner.assert(Array.isArray(jobs), 'Jobs list should be an array');
  });

  // 3. Identity and DID Tests
  runner.test('Identity', 'DID Registration', async () => {
    const didDocument = {
      '@context': 'https://www.w3.org/ns/did/v1',
      id: 'did:key:test123',
      publicKey: [{
        id: 'did:key:test123#key1',
        type: 'Ed25519VerificationKey2018',
        owner: 'did:key:test123',
        publicKeyBase58: 'H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV',
      }],
    };

    const registeredDid = await runner.client.registerDid(didDocument);
    await runner.assertResponseOk(registeredDid, 'registerDid');
    await runner.assert(typeof registeredDid === 'string', 'Registered DID should be a string');
    await runner.assert(registeredDid.startsWith('did:'), 'Registered DID should start with "did:"');
  });

  runner.test('Identity', 'DID Resolution', async () => {
    const testDid = 'did:key:test123';
    const resolvedDocument = await runner.client.resolveDid(testDid);
    await runner.assertResponseOk(resolvedDocument, 'resolveDid');
    await runner.assert(resolvedDocument.id === testDid, 'Resolved DID should match requested DID');
  });

  // 4. Governance Tests
  runner.test('Governance', 'Proposal Submission', async () => {
    const proposal = {
      title: 'Test Proposal',
      description: 'This is a test proposal for integration testing',
      proposer_did: 'did:key:testproposer',
      proposal: {
        type: 'GenericText',
        text: 'Test proposal content',
      },
      duration_secs: 3600,
    };

    const proposalId = await runner.client.submitProposal(proposal);
    await runner.assertResponseOk(proposalId, 'submitProposal');
    await runner.assert(typeof proposalId === 'string', 'Proposal ID should be a string');
  });

  runner.test('Governance', 'Vote Casting', async () => {
    // Submit a proposal first
    const proposal = {
      title: 'Vote Test Proposal',
      description: 'Proposal for testing vote casting',
      proposer_did: 'did:key:testproposer',
      proposal: {
        type: 'GenericText', 
        text: 'Vote on this test proposal',
      },
      duration_secs: 3600,
    };

    const proposalId = await runner.client.submitProposal(proposal);
    
    // Now cast a vote
    await runner.client.vote(proposalId, 'yes');
    // If no error is thrown, the vote was successfully cast
  });

  // 5. Federation Management Tests
  runner.test('Federation', 'Federation Status', async () => {
    const federationStatus = await runner.client.federation.getStatus();
    await runner.assertResponseOk(federationStatus, 'federation.getStatus');
    await runner.assert(federationStatus.name !== undefined, 'Federation should have a name');
  });

  runner.test('Federation', 'Peer Management', async () => {
    const peers = await runner.client.federation.listPeers();
    await runner.assertResponseOk(peers, 'federation.listPeers');
    await runner.assert(Array.isArray(peers), 'Peers list should be an array');
  });

  // 6. Account and Token Tests
  runner.test('Account', 'Mana Balance Check', async () => {
    const testDid = 'did:key:testaccount';
    const balance = await runner.client.account.getManaBalance(testDid);
    await runner.assertResponseOk(balance, 'account.getManaBalance');
    await runner.assert(typeof balance.balance === 'number', 'Mana balance should be a number');
  });

  runner.test('Tokens', 'Token Balance Retrieval', async () => {
    const testDid = 'did:key:testaccount';
    const balances = await runner.client.tokens.listBalances(testDid);
    await runner.assertResponseOk(balances, 'tokens.listBalances');
    await runner.assert(Array.isArray(balances), 'Token balances should be an array');
  });

  // 7. DAG Storage Tests
  runner.test('DAG Storage', 'DAG Root Status', async () => {
    const dagRoot = await runner.client.dag.getRoot();
    // DAG root can be null for new nodes, so just check it doesn't error
    await runner.assert(dagRoot === null || typeof dagRoot === 'string', 'DAG root should be null or string');
  });

  runner.test('DAG Storage', 'DAG Sync Status', async () => {
    const syncStatus = await runner.client.dag.getSyncStatus();
    await runner.assertResponseOk(syncStatus, 'dag.getSyncStatus');
    await runner.assert(typeof syncStatus.in_sync === 'boolean', 'Sync status should include in_sync boolean');
  });

  return await runner.run();
}

// Export for use in other test files
export { IntegrationTestRunner, TEST_CONFIG, MOCK_RESPONSES };

// Auto-run if executed directly
if (typeof require !== 'undefined' && require.main === module) {
  const useOfflineMode = process.argv.includes('--offline') || TEST_CONFIG.useOfflineMode;
  
  runIntegrationTests(useOfflineMode)
    .then(results => {
      console.log('\n🎉 Integration test suite completed!');
      if (typeof process !== 'undefined') {
        process.exit(results.failed > 0 ? 1 : 0);
      }
    })
    .catch(error => {
      console.error('❌ Integration test suite failed:', error);
      if (typeof process !== 'undefined') {
        process.exit(1);
      }
    });
}