/**
 * Comprehensive Error Handling Example
 * 
 * This example demonstrates:
 * - All error types and their handling
 * - Retry strategies for different error scenarios
 * - Graceful degradation and fallback mechanisms
 * - Error logging and monitoring
 * - Recovery patterns and best practices
 */

import { 
  ICNClient, 
  createStorage, 
  ICNError,
  ICNConnectionError,
  ICNAuthError,
  ICNValidationError,
  ICNNetworkError,
  ICNGovernanceError,
  ICNCredentialError,
  ICNTrustError,
  ICNMeshError,
  ICNStorageError,
  ICNTokenError,
  ICNTimeoutError,
  ICNRateLimitError,
  ErrorFactory,
  ErrorUtils
} from '@icn/ts-sdk';

async function errorHandlingExample() {
  console.log('🛡️  Starting Comprehensive Error Handling Example\n');

  // Initialize client with intentionally problematic settings for demonstration
  const client = new ICNClient({
    nodeEndpoint: 'http://localhost:8080', // May not be running
    network: 'devnet',
    timeout: 5000, // Short timeout for demo
    storage: createStorage('@error-example:'),
  });

  console.log('📋 Error Types and Handling Strategies:\n');

  // 1. Connection Error Handling
  console.log('🔌 Testing Connection Error Handling...');
  try {
    await client.connect();
    console.log('✅ Connection successful');
  } catch (error) {
    if (ErrorUtils.isErrorType(error, ICNConnectionError)) {
      console.log('❌ Connection Error detected');
      console.log(`   Message: ${error.message}`);
      console.log(`   Code: ${error.code}`);
      console.log('   Handling Strategy: Retry with exponential backoff');
      
      // Implement retry logic
      let retryCount = 0;
      const maxRetries = 3;
      
      while (retryCount < maxRetries) {
        retryCount++;
        const retryDelay = ErrorUtils.getRetryDelay(error, retryCount);
        
        console.log(`   Retry ${retryCount}/${maxRetries} in ${retryDelay}ms...`);
        await new Promise(resolve => setTimeout(resolve, retryDelay));
        
        try {
          await client.connect();
          console.log('✅ Connection successful on retry');
          break;
        } catch (retryError) {
          if (retryCount === maxRetries) {
            console.log('❌ All connection retries exhausted');
            console.log('   Fallback: Switch to offline mode or alternative endpoint');
          }
        }
      }
    } else {
      console.log('❌ Unexpected connection error:', ErrorUtils.getErrorMessage(error));
    }
  }
  console.log();

  // 2. Network Error Handling with Different Status Codes
  console.log('🌐 Testing Network Error Handling...');
  
  const networkErrorScenarios = [
    { status: 400, message: 'Bad Request', description: 'Validation error on server side' },
    { status: 401, message: 'Unauthorized', description: 'Authentication required' },
    { status: 403, message: 'Forbidden', description: 'Insufficient permissions' },
    { status: 404, message: 'Not Found', description: 'Resource does not exist' },
    { status: 408, message: 'Request Timeout', description: 'Server timeout' },
    { status: 429, message: 'Too Many Requests', description: 'Rate limit exceeded' },
    { status: 500, message: 'Internal Server Error', description: 'Server-side error' },
    { status: 502, message: 'Bad Gateway', description: 'Upstream server error' },
    { status: 503, message: 'Service Unavailable', description: 'Service temporarily down' }
  ];
  
  networkErrorScenarios.forEach(scenario => {
    const error = ErrorFactory.fromApiError(scenario.status, scenario.message, {}, 'http://example.com/api');
    
    console.log(`   Status ${scenario.status}: ${scenario.description}`);
    console.log(`     Error Type: ${error.constructor.name}`);
    console.log(`     Retryable: ${ErrorUtils.isRetryableError(error) ? '✅ Yes' : '❌ No'}`);
    
    if (ErrorUtils.isRetryableError(error)) {
      const retryDelay = ErrorUtils.getRetryDelay(error, 1);
      console.log(`     Retry Delay: ${retryDelay}ms`);
    }
    
    // Specific handling based on error type
    if (ErrorUtils.isErrorType(error, ICNAuthError)) {
      console.log('     Handling: Redirect to authentication, refresh tokens');
    } else if (ErrorUtils.isErrorType(error, ICNValidationError)) {
      console.log('     Handling: Show validation errors to user, fix input');
    } else if (ErrorUtils.isErrorType(error, ICNTimeoutError)) {
      console.log('     Handling: Retry with longer timeout, show progress indicator');
    } else if (ErrorUtils.isErrorType(error, ICNRateLimitError)) {
      console.log('     Handling: Implement exponential backoff, queue requests');
    } else if (ErrorUtils.isErrorType(error, ICNNetworkError)) {
      console.log('     Handling: Retry for 5xx errors, show error for 4xx errors');
    }
  });
  console.log();

  // 3. Validation Error Handling
  console.log('📝 Testing Validation Error Handling...');
  
  const validationScenarios = [
    { field: 'did', value: 'invalid-did', rule: 'DID format validation' },
    { field: 'amount', value: '-100', rule: 'Positive number validation' },
    { field: 'email', value: 'not-an-email', rule: 'Email format validation' },
    { field: 'url', value: 'not-a-url', rule: 'URL format validation' }
  ];
  
  validationScenarios.forEach(scenario => {
    const error = new ICNValidationError(
      `${scenario.field} is invalid: ${scenario.rule}`,
      scenario.field,
      { value: scenario.value, rule: scenario.rule }
    );
    
    console.log(`   Field: ${scenario.field}`);
    console.log(`     Invalid Value: ${scenario.value}`);
    console.log(`     Rule: ${scenario.rule}`);
    console.log(`     Error Message: ${error.message}`);
    console.log(`     Field Name: ${error.field}`);
    console.log('     Handling: Show field-specific error, highlight invalid input');
  });
  console.log();

  // 4. API-Specific Error Handling
  console.log('🔧 Testing API-Specific Error Handling...');
  
  // Governance errors
  try {
    // This will likely fail due to invalid parameters
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Proposal validation failed: Invalid proposer DID');
    }, 'Governance proposal submission');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('🏛️  Governance Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Validate proposer credentials, check proposal format');
    }
  }
  
  // Credential errors
  try {
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Credential verification failed: Expired credential');
    }, 'Credential verification');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('🆔 Credential Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Check credential expiration, request renewal');
    }
  }
  
  // Trust errors
  try {
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Trust relationship not found');
    }, 'Trust network operation');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('🤝 Trust Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Establish trust relationship, check entity existence');
    }
  }
  
  // Mesh errors
  try {
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Insufficient resources for job execution');
    }, 'Mesh job submission');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('⚙️  Mesh Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Reduce resource requirements, wait for availability');
    }
  }
  
  // Storage errors
  try {
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Storage quota exceeded');
    }, 'Storage operation');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('💾 Storage Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Clean up old data, request quota increase');
    }
  }
  
  // Token errors
  try {
    await ErrorUtils.wrapWithErrorHandling(async () => {
      throw new Error('Insufficient token balance');
    }, 'Token transfer');
  } catch (error) {
    if (error instanceof ICNError) {
      console.log('🪙 Token Error Context:');
      console.log(`     Message: ${error.message}`);
      console.log(`     Context: ${error.details?.context}`);
      console.log('     Handling: Check balance, request tokens, reduce amount');
    }
  }
  console.log();

  // 5. Timeout Error Handling
  console.log('⏰ Testing Timeout Error Handling...');
  
  const timeoutScenarios = [
    { operation: 'Node connection', timeout: 5000, strategy: 'Retry with longer timeout' },
    { operation: 'Large file upload', timeout: 30000, strategy: 'Resume upload, chunk file' },
    { operation: 'Complex computation', timeout: 60000, strategy: 'Check job status, extend timeout' },
    { operation: 'Network sync', timeout: 120000, strategy: 'Partial sync, background continuation' }
  ];
  
  timeoutScenarios.forEach(scenario => {
    const error = new ICNTimeoutError(
      `${scenario.operation} timed out after ${scenario.timeout}ms`,
      scenario.timeout
    );
    
    console.log(`   Operation: ${scenario.operation}`);
    console.log(`     Timeout: ${scenario.timeout}ms`);
    console.log(`     Strategy: ${scenario.strategy}`);
    console.log(`     Retryable: ${ErrorUtils.isRetryableError(error) ? 'Yes' : 'No'}`);
    
    if (ErrorUtils.isRetryableError(error)) {
      const retryDelay = ErrorUtils.getRetryDelay(error, 1);
      console.log(`     Retry Delay: ${retryDelay}ms`);
    }
  });
  console.log();

  // 6. Rate Limiting Error Handling
  console.log('🚦 Testing Rate Limiting Error Handling...');
  
  const rateLimitScenarios = [
    { retryAfter: 60, operation: 'API calls', strategy: 'Queue requests, show progress' },
    { retryAfter: 300, operation: 'File uploads', strategy: 'Pause uploads, resume later' },
    { retryAfter: 3600, operation: 'Bulk operations', strategy: 'Schedule for later, notify user' }
  ];
  
  rateLimitScenarios.forEach(scenario => {
    const error = new ICNRateLimitError(
      `Rate limit exceeded for ${scenario.operation}`,
      scenario.retryAfter
    );
    
    console.log(`   Operation: ${scenario.operation}`);
    console.log(`     Retry After: ${scenario.retryAfter}s`);
    console.log(`     Strategy: ${scenario.strategy}`);
    console.log(`     Calculated Delay: ${ErrorUtils.getRetryDelay(error, 1)}ms`);
  });
  console.log();

  // 7. Error Recovery Patterns
  console.log('🔄 Testing Error Recovery Patterns...');
  
  // Circuit Breaker Pattern
  class CircuitBreaker {
    private failures = 0;
    private lastFailureTime = 0;
    private state: 'closed' | 'open' | 'half-open' = 'closed';
    
    constructor(
      private maxFailures = 5,
      private resetTimeout = 60000 // 1 minute
    ) {}
    
    async execute<T>(operation: () => Promise<T>): Promise<T> {
      if (this.state === 'open') {
        if (Date.now() - this.lastFailureTime < this.resetTimeout) {
          throw new ICNError('CIRCUIT_BREAKER_OPEN', 'Circuit breaker is open');
        } else {
          this.state = 'half-open';
        }
      }
      
      try {
        const result = await operation();
        this.onSuccess();
        return result;
      } catch (error) {
        this.onFailure();
        throw error;
      }
    }
    
    private onSuccess() {
      this.failures = 0;
      this.state = 'closed';
    }
    
    private onFailure() {
      this.failures++;
      this.lastFailureTime = Date.now();
      
      if (this.failures >= this.maxFailures) {
        this.state = 'open';
      }
    }
    
    getState() {
      return {
        state: this.state,
        failures: this.failures,
        lastFailureTime: this.lastFailureTime
      };
    }
  }
  
  const circuitBreaker = new CircuitBreaker(3, 30000);
  
  console.log('🔌 Circuit Breaker Pattern:');
  
  // Simulate operations that may fail
  for (let i = 1; i <= 5; i++) {
    try {
      await circuitBreaker.execute(async () => {
        if (Math.random() > 0.6) { // 40% success rate
          return `Operation ${i} successful`;
        } else {
          throw new ICNNetworkError(`Operation ${i} failed`);
        }
      });
      
      console.log(`   ✅ Operation ${i}: Success`);
    } catch (error) {
      console.log(`   ❌ Operation ${i}: ${ErrorUtils.getErrorMessage(error)}`);
    }
    
    const state = circuitBreaker.getState();
    console.log(`      Circuit state: ${state.state} (failures: ${state.failures})`);
  }
  console.log();

  // 8. Graceful Degradation
  console.log('⬇️ Testing Graceful Degradation...');
  
  class ServiceManager {
    private services = new Map<string, boolean>();
    
    constructor() {
      this.services.set('governance', true);
      this.services.set('credentials', true);
      this.services.set('mesh', true);
      this.services.set('trust', true);
    }
    
    async callService<T>(serviceName: string, operation: () => Promise<T>, fallback?: () => T): Promise<T> {
      if (!this.services.get(serviceName)) {
        if (fallback) {
          console.log(`   🔄 Service ${serviceName} unavailable, using fallback`);
          return fallback();
        } else {
          throw new ICNError('SERVICE_UNAVAILABLE', `Service ${serviceName} is currently unavailable`);
        }
      }
      
      try {
        return await operation();
      } catch (error) {
        // Mark service as unavailable on failure
        this.services.set(serviceName, false);
        console.log(`   ⚠️  Service ${serviceName} failed, marking as unavailable`);
        
        if (fallback) {
          console.log(`   🔄 Using fallback for ${serviceName}`);
          return fallback();
        } else {
          throw error;
        }
      }
    }
    
    getServiceStatus() {
      return Object.fromEntries(this.services);
    }
  }
  
  const serviceManager = new ServiceManager();
  
  // Test graceful degradation
  const operationResults = await Promise.allSettled([
    serviceManager.callService(
      'governance',
      async () => {
        if (Math.random() > 0.5) throw new Error('Governance service error');
        return 'Governance data loaded';
      },
      () => 'Cached governance data'
    ),
    serviceManager.callService(
      'credentials',
      async () => {
        if (Math.random() > 0.5) throw new Error('Credentials service error');
        return 'Credentials verified';
      },
      () => 'Offline verification mode'
    ),
    serviceManager.callService(
      'mesh',
      async () => {
        if (Math.random() > 0.5) throw new Error('Mesh service error');
        return 'Job submitted to mesh';
      }
      // No fallback - will fail if service is down
    )
  ]);
  
  operationResults.forEach((result, index) => {
    const serviceName = ['governance', 'credentials', 'mesh'][index];
    if (result.status === 'fulfilled') {
      console.log(`   ✅ ${serviceName}: ${result.value}`);
    } else {
      console.log(`   ❌ ${serviceName}: ${ErrorUtils.getErrorMessage(result.reason)}`);
    }
  });
  
  console.log('   Service Status:', serviceManager.getServiceStatus());
  console.log();

  // 9. Error Logging and Monitoring
  console.log('📊 Error Logging and Monitoring...');
  
  class ErrorLogger {
    private errors: Array<{
      timestamp: number;
      type: string;
      message: string;
      context?: string;
      details?: any;
    }> = [];
    
    log(error: unknown, context?: string) {
      let errorInfo;
      
      if (error instanceof ICNError) {
        errorInfo = {
          timestamp: Date.now(),
          type: error.constructor.name,
          message: error.message,
          context: context || error.details?.context,
          details: error.details
        };
      } else {
        errorInfo = {
          timestamp: Date.now(),
          type: 'UnknownError',
          message: ErrorUtils.getErrorMessage(error),
          context,
          details: { originalError: error }
        };
      }
      
      this.errors.push(errorInfo);
      
      // In real application, send to monitoring service
      console.log(`   📝 Logged: ${errorInfo.type} - ${errorInfo.message}`);
    }
    
    getErrorStats() {
      const totalErrors = this.errors.length;
      const errorsByType = this.errors.reduce((acc, error) => {
        acc[error.type] = (acc[error.type] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);
      
      const recentErrors = this.errors.filter(
        error => Date.now() - error.timestamp < 3600000 // Last hour
      ).length;
      
      return {
        totalErrors,
        errorsByType,
        recentErrors,
        errorRate: recentErrors / 3600 // errors per second in last hour
      };
    }
    
    getRecentErrors(count = 5) {
      return this.errors
        .slice(-count)
        .map(error => ({
          type: error.type,
          message: error.message,
          timestamp: new Date(error.timestamp).toISOString(),
          context: error.context
        }));
    }
  }
  
  const errorLogger = new ErrorLogger();
  
  // Simulate various errors for logging
  const testErrors = [
    new ICNConnectionError('Connection timeout'),
    new ICNValidationError('Invalid DID format', 'did'),
    new ICNNetworkError('Service unavailable', 503),
    new ICNTokenError('Insufficient balance'),
    new ICNTimeoutError('Operation timeout', 30000)
  ];
  
  testErrors.forEach((error, index) => {
    errorLogger.log(error, `Test operation ${index + 1}`);
  });
  
  const stats = errorLogger.getErrorStats();
  console.log('   📊 Error Statistics:');
  console.log(`     Total Errors: ${stats.totalErrors}`);
  console.log(`     Recent Errors (1h): ${stats.recentErrors}`);
  console.log(`     Error Rate: ${stats.errorRate.toFixed(4)}/sec`);
  console.log('     Errors by Type:');
  Object.entries(stats.errorsByType).forEach(([type, count]) => {
    console.log(`       ${type}: ${count}`);
  });
  
  console.log('   📋 Recent Errors:');
  errorLogger.getRecentErrors(3).forEach((error, index) => {
    console.log(`     ${index + 1}. ${error.type}: ${error.message}`);
    console.log(`        Context: ${error.context || 'None'}`);
    console.log(`        Time: ${error.timestamp}`);
  });
  console.log();

  // 10. Best Practices Summary
  console.log('💡 Error Handling Best Practices Summary:\n');
  
  console.log('🔧 Implementation Strategies:');
  console.log('   • Use specific error types for different failure scenarios');
  console.log('   • Implement retry logic with exponential backoff for transient errors');
  console.log('   • Use circuit breakers to prevent cascading failures');
  console.log('   • Provide graceful degradation with fallback mechanisms');
  console.log('   • Log errors with context for debugging and monitoring');
  console.log('   • Show user-friendly error messages while logging technical details');
  console.log('   • Validate input early to prevent downstream errors');
  console.log('   • Handle rate limiting gracefully with request queuing');
  
  console.log('\n🎯 User Experience Guidelines:');
  console.log('   • Show progress indicators for long-running operations');
  console.log('   • Provide clear, actionable error messages');
  console.log('   • Offer retry options for recoverable errors');
  console.log('   • Implement offline mode for network-dependent features');
  console.log('   • Cache data to reduce dependency on external services');
  console.log('   • Provide help links or contact information for complex errors');
  
  console.log('\n📊 Monitoring and Alerting:');
  console.log('   • Track error rates and patterns over time');
  console.log('   • Set up alerts for critical error thresholds');
  console.log('   • Monitor service health and availability');
  console.log('   • Correlate errors with system performance metrics');
  console.log('   • Implement distributed tracing for complex operations');
  console.log('   • Regular error log analysis and pattern detection');

  console.log('\n🎉 Comprehensive Error Handling example completed successfully!');
  console.log('\n🛡️  Key Benefits:');
  console.log('   • Improved application reliability and user experience');
  console.log('   • Faster error diagnosis and resolution');
  console.log('   • Reduced system downtime through graceful degradation');
  console.log('   • Better monitoring and alerting capabilities');
  console.log('   • Enhanced debugging and troubleshooting tools');

  try {
    await client.disconnect();
    console.log('\n🔌 Disconnected from ICN node');
  } catch (error) {
    console.log('\n⚠️  Disconnect error (non-critical):', ErrorUtils.getErrorMessage(error));
  }
}

// Run the example
if (require.main === module) {
  errorHandlingExample().catch(console.error);
}

export { errorHandlingExample };