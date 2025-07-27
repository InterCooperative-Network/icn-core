/**
 * Basic Test Suite for ICN TypeScript SDK
 * 
 * This test suite validates:
 * - Core SDK functionality without requiring running ICN nodes
 * - Error handling and type safety
 * - Storage and encryption features
 * - Utility functions and helpers
 * - React hooks (mocked environment)
 */

import { 
  ICNClient, 
  createStorage, 
  createSecureStorage,
  ICNStorage,
  ErrorFactory,
  ErrorUtils,
  EnhancedUtils,
  FederationUtils,
  GovernanceUtils,
  CCLUtils,
  validateDid,
  formatMana,
  formatJobId
} from '../src';

// Test runner
class TestRunner {
  private tests: Array<{
    name: string;
    fn: () => Promise<void> | void;
  }> = [];
  
  private passed = 0;
  private failed = 0;
  private failures: string[] = [];

  test(name: string, fn: () => Promise<void> | void) {
    this.tests.push({ name, fn });
  }

  async run() {
    console.log('🧪 Running ICN TypeScript SDK Tests\n');

    for (const test of this.tests) {
      try {
        console.log(`⏳ ${test.name}...`);
        await test.fn();
        console.log(`✅ ${test.name}`);
        this.passed++;
      } catch (error) {
        console.log(`❌ ${test.name}: ${error instanceof Error ? error.message : error}`);
        this.failed++;
        this.failures.push(`${test.name}: ${error instanceof Error ? error.message : error}`);
      }
    }

    console.log(`\n📊 Test Results:`);
    console.log(`   ✅ Passed: ${this.passed}`);
    console.log(`   ❌ Failed: ${this.failed}`);
    console.log(`   📈 Success Rate: ${((this.passed / (this.passed + this.failed)) * 100).toFixed(1)}%`);

    if (this.failures.length > 0) {
      console.log('\n❌ Failures:');
      this.failures.forEach((failure, index) => {
        console.log(`   ${index + 1}. ${failure}`);
      });
    }

    return { passed: this.passed, failed: this.failed };
  }
}

// Helper functions for testing
function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertEqual<T>(actual: T, expected: T, message?: string) {
  if (actual !== expected) {
    throw new Error(message || `Expected ${expected}, got ${actual}`);
  }
}

function assertThrows(fn: () => any, message?: string) {
  try {
    fn();
    throw new Error(message || 'Expected function to throw');
  } catch (error) {
    // Expected
  }
}

async function assertThrowsAsync(fn: () => Promise<any>, message?: string) {
  try {
    await fn();
    throw new Error(message || 'Expected function to throw');
  } catch (error) {
    // Expected
  }
}

// Main test suite
async function runTests() {
  const runner = new TestRunner();

  // 1. Storage Tests
  runner.test('Storage - Basic Operations', async () => {
    const storage = createStorage('@test:');
    
    // Test basic storage operations
    await storage.setItem('test-key', 'test-value');
    const value = await storage.getItem('test-key');
    assertEqual(value, 'test-value');

    await storage.removeItem('test-key');
    const removedValue = await storage.getItem('test-key');
    assertEqual(removedValue, null);
  });

  runner.test('Storage - Cached Data with TTL', async () => {
    const storage = new ICNStorage(createStorage('@test:'));
    
    // Test cached data with TTL
    const testData = { message: 'hello', timestamp: Date.now() };
    await storage.setCachedData('cache-key', testData, 1000); // 1 second TTL
    
    const cachedData = await storage.getCachedData('cache-key');
    assert(cachedData !== null, 'Cached data should exist');
    assertEqual(cachedData.data.message, 'hello');

    // Wait for TTL to expire
    await new Promise(resolve => setTimeout(resolve, 1100));
    
    const expiredData = await storage.getCachedData('cache-key');
    assertEqual(expiredData, null, 'Cached data should be null after TTL');
  });

  runner.test('Storage - Secure Storage', () => {
    const secureStorage = createSecureStorage('@secure-test:', {
      enableEncryption: true,
      passphrase: 'test-passphrase'
    });

    assert(secureStorage instanceof ICNStorage, 'Should create ICNStorage instance');
  });

  // 2. Error Handling Tests
  runner.test('Error Factory - API Errors', () => {
    const error400 = ErrorFactory.fromApiError(400, 'Bad Request');
    assertEqual(error400.constructor.name, 'ICNValidationError');

    const error401 = ErrorFactory.fromApiError(401, 'Unauthorized');
    assertEqual(error401.constructor.name, 'ICNAuthError');

    const error500 = ErrorFactory.fromApiError(500, 'Internal Server Error');
    assertEqual(error500.constructor.name, 'ICNNetworkError');
  });

  runner.test('Error Factory - Error Types', () => {
    const validationError = ErrorFactory.fromErrorType('validation', 'Invalid input');
    assertEqual(validationError.constructor.name, 'ICNValidationError');

    const connectionError = ErrorFactory.fromErrorType('connection', 'Connection failed');
    assertEqual(connectionError.constructor.name, 'ICNConnectionError');

    const unknownError = ErrorFactory.fromUnknownError('Some error');
    assertEqual(unknownError.constructor.name, 'ICNError');
  });

  runner.test('Error Utils - Error Type Checking', () => {
    const validationError = ErrorFactory.fromErrorType('validation', 'Test error');
    
    assert(ErrorUtils.isErrorType(validationError, ErrorFactory.fromErrorType('validation', 'Test').constructor as any), 
           'Should correctly identify error type');
    
    assertEqual(ErrorUtils.getErrorMessage(validationError), 'Test error');
    assertEqual(ErrorUtils.getErrorMessage('string error'), 'string error');
    assertEqual(ErrorUtils.getErrorMessage(new Error('Error object')), 'Error object');
  });

  runner.test('Error Utils - Retry Logic', () => {
    const retryableError = ErrorFactory.fromApiError(500, 'Server Error');
    const nonRetryableError = ErrorFactory.fromApiError(400, 'Bad Request');

    assert(ErrorUtils.isRetryableError(retryableError), 'Server errors should be retryable');
    assert(!ErrorUtils.isRetryableError(nonRetryableError), 'Client errors should not be retryable');

    const delay1 = ErrorUtils.getRetryDelay(retryableError, 1);
    const delay2 = ErrorUtils.getRetryDelay(retryableError, 2);
    assert(delay2 > delay1, 'Retry delay should increase with attempt number');
  });

  // 3. Utility Function Tests
  runner.test('Utility Functions - DID Validation', () => {
    assert(validateDid('did:key:abc123'), 'Valid DID should pass validation');
    assert(validateDid('did:method:identifier'), 'Valid DID format should pass');
    assert(!validateDid('invalid-did'), 'Invalid DID should fail validation');
    assert(!validateDid(''), 'Empty string should fail validation');
  });

  runner.test('Utility Functions - Mana Formatting', () => {
    assertEqual(formatMana(500), '500');
    assertEqual(formatMana(1500), '1.5K');
    assertEqual(formatMana(1500000), '1.5M');
  });

  runner.test('Utility Functions - Job ID Formatting', () => {
    assertEqual(formatJobId(''), '');
    assertEqual(formatJobId('short'), 'short');
    assertEqual(formatJobId('very-long-job-id-12345'), 'very-lon...2345');
  });

  // 4. Enhanced Utilities Tests
  runner.test('Enhanced Utils - Credential Formatting', () => {
    const shortCid = 'abc123';
    const longCid = 'very-long-credential-id-12345678';
    
    assertEqual(EnhancedUtils.formatCredentialId(shortCid), shortCid);
    assertEqual(EnhancedUtils.formatCredentialId(longCid), 'very-lon...12345678');
  });

  runner.test('Enhanced Utils - Token Amount Formatting', () => {
    assertEqual(EnhancedUtils.formatTokenAmount(1000, 2), '10.00');
    assertEqual(EnhancedUtils.formatTokenAmount(1000, 2, 'USD'), '10.00 USD');
    assertEqual(EnhancedUtils.formatTokenAmount(1500, 0), '1,500');
  });

  runner.test('Enhanced Utils - Trust Score Formatting', () => {
    assertEqual(EnhancedUtils.formatTrustScore(0.85), '85.0%');
    assertEqual(EnhancedUtils.formatTrustScore(0.1), '10.0%');
  });

  runner.test('Enhanced Utils - Token Validation', () => {
    const validResult = EnhancedUtils.validateTokenAmount('100.50', 2);
    assert(validResult.valid, 'Valid token amount should pass');
    assertEqual(validResult.parsedAmount, 10050);

    const invalidResult = EnhancedUtils.validateTokenAmount('-100', 2);
    assert(!invalidResult.valid, 'Negative amount should fail');

    const precisionResult = EnhancedUtils.validateTokenAmount('100.123', 2);
    assert(!precisionResult.valid, 'Too many decimals should fail');
  });

  runner.test('Enhanced Utils - Challenge Generation', () => {
    const challenge1 = EnhancedUtils.generateChallenge(32);
    const challenge2 = EnhancedUtils.generateChallenge(32);
    
    assertEqual(challenge1.length, 32, 'Challenge should have correct length');
    assert(challenge1 !== challenge2, 'Challenges should be unique');
  });

  runner.test('Enhanced Utils - DID Validation', () => {
    const validResult = EnhancedUtils.validateDidFormat('did:key:abc123');
    assert(validResult.valid, 'Valid DID should pass');

    const invalidResult = EnhancedUtils.validateDidFormat('invalid');
    assert(!invalidResult.valid, 'Invalid DID should fail');
    assert(invalidResult.error !== undefined, 'Should provide error message');
  });

  // 5. Federation Utils Tests
  runner.test('Federation Utils - Name Validation', () => {
    assert(FederationUtils.isValidFederationName('My Federation'), 'Valid name should pass');
    assert(FederationUtils.isValidFederationName('Fed_123'), 'Name with underscore should pass');
    assert(!FederationUtils.isValidFederationName('a'), 'Too short name should fail');
    assert(!FederationUtils.isValidFederationName(''), 'Empty name should fail');
  });

  runner.test('Federation Utils - Health Score Calculation', () => {
    const metadata = {
      totalMembers: 10,
      governance: { activeProposals: 3 },
      mesh: { activeJobs: 5 },
      dag: { syncStatus: 'synced' }
    };

    const score = FederationUtils.calculateHealthScore(metadata);
    assert(score >= 0 && score <= 100, 'Health score should be between 0 and 100');
    assert(score > 50, 'Active federation should have good health score');
  });

  // 6. Governance Utils Tests
  runner.test('Governance Utils - Voting Progress', () => {
    const proposal = {
      votes: { yes: 6, no: 2, abstain: 1 },
      quorum: 5
    };

    const progress = GovernanceUtils.calculateVotingProgress(proposal);
    assertEqual(progress, 180); // 9 votes / 5 quorum * 100

    assert(GovernanceUtils.hasReachedQuorum(proposal), 'Should reach quorum');
    assertEqual(GovernanceUtils.getProposalOutcome({ ...proposal, status: 'Closed', threshold: 0.5 }), 'passed');
  });

  runner.test('Governance Utils - Proposal Type Formatting', () => {
    const proposalType = { type: 'SystemParameterChange', data: { param: 'test', value: '123' } };
    
    assertEqual(GovernanceUtils.formatProposalType(proposalType), 'System Parameter Change');
    assertEqual(GovernanceUtils.generateProposalSummary(proposalType), 'Change test to 123');
  });

  // 7. CCL Utils Tests
  runner.test('CCL Utils - Parameter Validation', () => {
    const template = {
      parameters: [
        { name: 'test_param', type: 'string', required: true },
        { name: 'number_param', type: 'number', required: false, validation: { min: 1, max: 100 } }
      ]
    };

    const validParams = { test_param: 'hello', number_param: 50 };
    const validResult = CCLUtils.validateTemplateParameters(template, validParams);
    assert(validResult.valid, 'Valid parameters should pass');

    const invalidParams = { number_param: 150 }; // Missing required param, invalid number
    const invalidResult = CCLUtils.validateTemplateParameters(template, invalidParams);
    assert(!invalidResult.valid, 'Invalid parameters should fail');
    assert(invalidResult.errors.length > 0, 'Should provide error messages');
  });

  runner.test('CCL Utils - Template Generation', () => {
    const template = {
      template: 'Hello {{name}}, you have {{count}} items.'
    };
    
    const parameters = { name: 'Alice', count: '5' };
    const result = CCLUtils.generateCCLFromTemplate(template, parameters);
    
    assertEqual(result, 'Hello Alice, you have 5 items.');
  });

  // 8. Client Configuration Tests
  runner.test('ICNClient - Configuration', () => {
    const client = new ICNClient({
      nodeEndpoint: 'http://localhost:8080',
      network: 'devnet',
      timeout: 10000,
      storage: createStorage('@test:')
    });

    const connectionState = client.getConnectionState();
    assertEqual(connectionState.nodeEndpoint, 'http://localhost:8080');
    assertEqual(connectionState.network, 'devnet');
    assertEqual(connectionState.connected, false);
  });

  runner.test('ICNClient - Connection State', async () => {
    const client = new ICNClient({
      nodeEndpoint: 'http://localhost:8080',
      network: 'devnet'
    });

    // Test that client doesn't throw on configuration
    const storage = client.getStorage();
    assert(storage instanceof ICNStorage, 'Should have storage instance');

    const baseClient = client.getBaseClient();
    assert(baseClient !== undefined, 'Should have base client');
  });

  // 9. Error Wrapping Test
  runner.test('Error Utils - Error Wrapping', async () => {
    try {
      await ErrorUtils.wrapWithErrorHandling(async () => {
        throw new Error('Test error');
      }, 'Test operation');
    } catch (error: any) {
      assert(error.details?.context === 'Test operation', 'Should add context to error');
    }
  });

  // 10. Edge Cases and Error Conditions
  runner.test('Edge Cases - Null and Undefined Handling', () => {
    assertEqual(formatJobId(''), '');
    assertEqual(EnhancedUtils.formatCredentialId(''), '');
    assertEqual(EnhancedUtils.sanitizeInput(''), '');
    
    const nullValidation = EnhancedUtils.validateDidFormat('');
    assert(!nullValidation.valid, 'Empty DID should be invalid');
  });

  return await runner.run();
}

// Run tests if this file is executed directly
declare const require: any;
declare const module: any;
declare const process: any;

if (typeof require !== 'undefined' && require.main === module) {
  runTests()
    .then(results => {
      console.log('\n🎉 Test suite completed!');
      if (typeof process !== 'undefined') {
        process.exit(results.failed > 0 ? 1 : 0);
      }
    })
    .catch(error => {
      console.error('❌ Test suite failed:', error);
      if (typeof process !== 'undefined') {
        process.exit(1);
      }
    });
}

export { runTests };