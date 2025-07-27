/**
 * Production Configuration Example
 * 
 * This example demonstrates:
 * - Production-ready client configuration
 * - Environment-specific settings
 * - Security best practices
 * - Performance optimization
 * - Monitoring and logging setup
 * - Connection pooling and resource management
 */

import { 
  ICNClient, 
  createStorage, 
  createSecureStorage,
  ICNConfig,
  createConfig,
  ErrorUtils,
  EnhancedUtils
} from '@icn/ts-sdk';

// Environment configuration interface
interface EnvironmentConfig {
  name: 'development' | 'staging' | 'production';
  nodeEndpoints: string[];
  timeout: number;
  retryAttempts: number;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  enableMetrics: boolean;
  storageEncryption: boolean;
  maxConcurrentRequests: number;
  circuitBreakerThreshold: number;
}

// Environment configurations
const environments: Record<string, EnvironmentConfig> = {
  development: {
    name: 'development',
    nodeEndpoints: ['http://localhost:8080'],
    timeout: 30000,
    retryAttempts: 2,
    logLevel: 'debug',
    enableMetrics: true,
    storageEncryption: false,
    maxConcurrentRequests: 10,
    circuitBreakerThreshold: 5
  },
  staging: {
    name: 'staging',
    nodeEndpoints: [
      'https://staging-node-1.icn.network',
      'https://staging-node-2.icn.network'
    ],
    timeout: 20000,
    retryAttempts: 3,
    logLevel: 'info',
    enableMetrics: true,
    storageEncryption: true,
    maxConcurrentRequests: 20,
    circuitBreakerThreshold: 10
  },
  production: {
    name: 'production',
    nodeEndpoints: [
      'https://node-1.icn.network',
      'https://node-2.icn.network',
      'https://node-3.icn.network'
    ],
    timeout: 15000,
    retryAttempts: 5,
    logLevel: 'warn',
    enableMetrics: true,
    storageEncryption: true,
    maxConcurrentRequests: 50,
    circuitBreakerThreshold: 20
  }
};

// Logger interface for production use
interface Logger {
  debug(message: string, meta?: any): void;
  info(message: string, meta?: any): void;
  warn(message: string, meta?: any): void;
  error(message: string, error?: Error, meta?: any): void;
}

// Production logger implementation
class ProductionLogger implements Logger {
  constructor(private level: string = 'info') {}

  private shouldLog(level: string): boolean {
    const levels = ['debug', 'info', 'warn', 'error'];
    return levels.indexOf(level) >= levels.indexOf(this.level);
  }

  private formatMessage(level: string, message: string, meta?: any): string {
    const timestamp = new Date().toISOString();
    const metaStr = meta ? ` ${JSON.stringify(meta)}` : '';
    return `[${timestamp}] ${level.toUpperCase()}: ${message}${metaStr}`;
  }

  debug(message: string, meta?: any): void {
    if (this.shouldLog('debug')) {
      console.debug(this.formatMessage('debug', message, meta));
    }
  }

  info(message: string, meta?: any): void {
    if (this.shouldLog('info')) {
      console.info(this.formatMessage('info', message, meta));
    }
  }

  warn(message: string, meta?: any): void {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', message, meta));
    }
  }

  error(message: string, error?: Error, meta?: any): void {
    if (this.shouldLog('error')) {
      const errorMeta = error ? { ...meta, error: error.stack } : meta;
      console.error(this.formatMessage('error', message, errorMeta));
    }
  }
}

// Metrics collector for monitoring
class MetricsCollector {
  private metrics = new Map<string, number>();
  private counters = new Map<string, number>();
  private timers = new Map<string, number>();

  increment(metric: string, value = 1): void {
    this.counters.set(metric, (this.counters.get(metric) || 0) + value);
  }

  gauge(metric: string, value: number): void {
    this.metrics.set(metric, value);
  }

  startTimer(metric: string): string {
    const timerId = `${metric}_${Date.now()}_${Math.random()}`;
    this.timers.set(timerId, Date.now());
    return timerId;
  }

  endTimer(timerId: string): number {
    const startTime = this.timers.get(timerId);
    if (startTime) {
      const duration = Date.now() - startTime;
      this.timers.delete(timerId);
      return duration;
    }
    return 0;
  }

  histogram(metric: string, value: number): void {
    // In production, this would use a proper histogram implementation
    this.metrics.set(`${metric}_current`, value);
    this.increment(`${metric}_count`);
  }

  getMetrics(): Record<string, number> {
    return {
      ...Object.fromEntries(this.metrics),
      ...Object.fromEntries(this.counters)
    };
  }

  reset(): void {
    this.metrics.clear();
    this.counters.clear();
    this.timers.clear();
  }
}

// Connection pool for managing multiple node connections
class ConnectionPool {
  private connections = new Map<string, ICNClient>();
  private activeConnections = new Set<string>();
  private failedConnections = new Set<string>();
  private currentIndex = 0;

  constructor(
    private endpoints: string[],
    private config: Partial<any>,
    private logger: Logger,
    private metrics: MetricsCollector
  ) {}

  async initialize(): Promise<void> {
    this.logger.info('Initializing connection pool', { endpoints: this.endpoints });

    for (const endpoint of this.endpoints) {
      try {
        const client = new ICNClient({
          ...this.config,
          nodeEndpoint: endpoint
        });

        await client.connect();
        this.connections.set(endpoint, client);
        this.activeConnections.add(endpoint);
        this.metrics.increment('connection_pool.successful_connections');
        
        this.logger.debug('Connection established', { endpoint });
      } catch (error) {
        this.failedConnections.add(endpoint);
        this.metrics.increment('connection_pool.failed_connections');
        
        this.logger.warn('Failed to connect to endpoint', { 
          endpoint, 
          error: ErrorUtils.getErrorMessage(error) 
        });
      }
    }

    if (this.activeConnections.size === 0) {
      throw new Error('No active connections available in pool');
    }

    this.logger.info('Connection pool initialized', {
      active: this.activeConnections.size,
      failed: this.failedConnections.size
    });
  }

  getConnection(): ICNClient {
    const activeEndpoints = Array.from(this.activeConnections);
    
    if (activeEndpoints.length === 0) {
      throw new Error('No active connections available');
    }

    // Round-robin load balancing
    const endpoint = activeEndpoints[this.currentIndex % activeEndpoints.length];
    this.currentIndex++;

    const client = this.connections.get(endpoint);
    if (!client) {
      throw new Error(`Connection not found for endpoint: ${endpoint}`);
    }

    this.metrics.increment('connection_pool.requests');
    return client;
  }

  async checkHealth(): Promise<void> {
    this.logger.debug('Checking connection pool health');

    for (const endpoint of this.endpoints) {
      try {
        const client = this.connections.get(endpoint);
        if (client) {
          // Test connection with a simple system call
          await client.system.getHealth();
          
          if (this.failedConnections.has(endpoint)) {
            this.failedConnections.delete(endpoint);
            this.activeConnections.add(endpoint);
            this.logger.info('Connection recovered', { endpoint });
            this.metrics.increment('connection_pool.recovered_connections');
          }
        }
      } catch (error) {
        if (this.activeConnections.has(endpoint)) {
          this.activeConnections.delete(endpoint);
          this.failedConnections.add(endpoint);
          this.logger.warn('Connection failed health check', { 
            endpoint, 
            error: ErrorUtils.getErrorMessage(error) 
          });
          this.metrics.increment('connection_pool.health_check_failures');
        }
      }
    }

    this.metrics.gauge('connection_pool.active_connections', this.activeConnections.size);
    this.metrics.gauge('connection_pool.failed_connections', this.failedConnections.size);
  }

  async disconnect(): Promise<void> {
    this.logger.info('Disconnecting connection pool');

    for (const [endpoint, client] of this.connections) {
      try {
        await client.disconnect();
        this.logger.debug('Connection closed', { endpoint });
      } catch (error) {
        this.logger.warn('Error disconnecting', { 
          endpoint, 
          error: ErrorUtils.getErrorMessage(error) 
        });
      }
    }

    this.connections.clear();
    this.activeConnections.clear();
    this.failedConnections.clear();
  }

  getStatus() {
    return {
      total: this.endpoints.length,
      active: this.activeConnections.size,
      failed: this.failedConnections.size,
      activeEndpoints: Array.from(this.activeConnections),
      failedEndpoints: Array.from(this.failedConnections)
    };
  }
}

// Production-ready ICN client wrapper
class ProductionICNClient {
  private connectionPool?: ConnectionPool;
  private logger: Logger;
  private metrics: MetricsCollector;
  private config: EnvironmentConfig;
  private healthCheckInterval?: NodeJS.Timeout;

  constructor(environment: string = 'production') {
    this.config = environments[environment] || environments.production;
    this.logger = new ProductionLogger(this.config.logLevel);
    this.metrics = new MetricsCollector();

    this.logger.info('Initializing ProductionICNClient', { 
      environment,
      config: this.config 
    });
  }

  async initialize(): Promise<void> {
    try {
      // Create secure storage
      const storage = this.config.storageEncryption 
        ? createSecureStorage(`@icn-${this.config.name}:`, {
            enableEncryption: true,
            passphrase: process.env.ICN_STORAGE_PASSPHRASE || 'default-passphrase'
          })
        : createStorage(`@icn-${this.config.name}:`);

      // Initialize connection pool
      this.connectionPool = new ConnectionPool(
        this.config.nodeEndpoints,
        {
          network: this.config.name === 'production' ? 'mainnet' : this.config.name,
          timeout: this.config.timeout,
          storage,
          encryptionConfig: {
            enableEncryption: this.config.storageEncryption
          }
        },
        this.logger,
        this.metrics
      );

      await this.connectionPool.initialize();

      // Start health check interval
      this.healthCheckInterval = setInterval(
        () => this.connectionPool?.checkHealth(),
        60000 // Check every minute
      );

      this.logger.info('ProductionICNClient initialized successfully');
      this.metrics.increment('client.initialization.success');

    } catch (error) {
      this.logger.error('Failed to initialize ProductionICNClient', error as Error);
      this.metrics.increment('client.initialization.failure');
      throw error;
    }
  }

  // Wrapper methods with metrics and error handling
  async withMetrics<T>(
    operation: string,
    fn: (client: ICNClient) => Promise<T>
  ): Promise<T> {
    if (!this.connectionPool) {
      throw new Error('Client not initialized');
    }

    const timer = this.metrics.startTimer(`operation.${operation}.duration`);
    this.metrics.increment(`operation.${operation}.attempts`);

    let lastError: Error | undefined;
    
    for (let attempt = 1; attempt <= this.config.retryAttempts; attempt++) {
      try {
        const client = this.connectionPool.getConnection();
        const result = await fn(client);
        
        const duration = this.metrics.endTimer(timer);
        this.metrics.histogram(`operation.${operation}.duration`, duration);
        this.metrics.increment(`operation.${operation}.success`);
        
        this.logger.debug('Operation completed successfully', {
          operation,
          attempt,
          duration
        });

        return result;

      } catch (error) {
        lastError = error as Error;
        this.metrics.increment(`operation.${operation}.failure`);
        
        this.logger.warn('Operation failed', {
          operation,
          attempt,
          maxAttempts: this.config.retryAttempts,
          error: ErrorUtils.getErrorMessage(error)
        });

        if (attempt < this.config.retryAttempts && ErrorUtils.isRetryableError(error)) {
          const delay = ErrorUtils.getRetryDelay(error, attempt);
          this.logger.debug('Retrying operation', { operation, attempt, delay });
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }

    this.metrics.endTimer(timer);
    throw lastError;
  }

  // Governance operations
  async submitProposal(request: any) {
    return this.withMetrics('governance.submitProposal', async (client) => {
      return await client.governance.submitProposal(request);
    });
  }

  async listProposals() {
    return this.withMetrics('governance.listProposals', async (client) => {
      return await client.governance.listProposals();
    });
  }

  async castVote(request: any) {
    return this.withMetrics('governance.castVote', async (client) => {
      return await client.governance.castVote(request);
    });
  }

  // Credential operations
  async issueCredential(request: any) {
    return this.withMetrics('credentials.issue', async (client) => {
      return await client.credentials.issueCredential(request);
    });
  }

  async verifyCredential(request: any) {
    return this.withMetrics('credentials.verify', async (client) => {
      return await client.credentials.verifyCredential(request);
    });
  }

  // Trust operations
  async getTrustScore(did: string) {
    return this.withMetrics('trust.getScore', async (client) => {
      return await client.trust.getTrustScore(did);
    });
  }

  async updateTrustRelationship(request: any) {
    return this.withMetrics('trust.updateRelationship', async (client) => {
      return await client.trust.updateTrustRelationship(request);
    });
  }

  // Mesh operations
  async submitJob(request: any) {
    return this.withMetrics('mesh.submitJob', async (client) => {
      return await client.mesh.submitJob(request);
    });
  }

  async getJobStatus(jobId: string) {
    return this.withMetrics('mesh.getJobStatus', async (client) => {
      return await client.mesh.getJobStatus(jobId);
    });
  }

  // Token operations
  async transferTokens(request: any) {
    return this.withMetrics('tokens.transfer', async (client) => {
      return await client.tokens.transferTokens(request);
    });
  }

  async getTokenBalances(did: string) {
    return this.withMetrics('tokens.getBalances', async (client) => {
      return await client.tokens.listBalances(did);
    });
  }

  // System operations
  async getSystemInfo() {
    return this.withMetrics('system.getInfo', async (client) => {
      return await client.system.getInfo();
    });
  }

  async getSystemHealth() {
    return this.withMetrics('system.getHealth', async (client) => {
      return await client.system.getHealth();
    });
  }

  // Monitoring and diagnostics
  getMetrics(): Record<string, number> {
    return this.metrics.getMetrics();
  }

  getConnectionStatus() {
    return this.connectionPool?.getStatus() || { error: 'Not initialized' };
  }

  async performHealthCheck(): Promise<{
    status: 'healthy' | 'degraded' | 'unhealthy';
    connections: any;
    metrics: Record<string, number>;
    timestamp: string;
  }> {
    const connections = this.getConnectionStatus();
    const metrics = this.getMetrics();
    
    let status: 'healthy' | 'degraded' | 'unhealthy' = 'healthy';
    
    if (connections.active === 0) {
      status = 'unhealthy';
    } else if (connections.active < connections.total / 2) {
      status = 'degraded';
    }

    return {
      status,
      connections,
      metrics,
      timestamp: new Date().toISOString()
    };
  }

  async shutdown(): Promise<void> {
    this.logger.info('Shutting down ProductionICNClient');

    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
    }

    if (this.connectionPool) {
      await this.connectionPool.disconnect();
    }

    // Export final metrics
    const finalMetrics = this.getMetrics();
    this.logger.info('Final metrics', finalMetrics);

    this.logger.info('ProductionICNClient shutdown complete');
  }
}

async function productionConfigExample() {
  console.log('🏭 Starting Production Configuration Example\n');

  // 1. Environment Configuration
  console.log('⚙️  Environment Configuration...');
  
  Object.entries(environments).forEach(([name, config]) => {
    console.log(`\n📋 ${name.toUpperCase()} Environment:`);
    console.log(`   Node Endpoints: ${config.nodeEndpoints.join(', ')}`);
    console.log(`   Timeout: ${config.timeout}ms`);
    console.log(`   Retry Attempts: ${config.retryAttempts}`);
    console.log(`   Log Level: ${config.logLevel}`);
    console.log(`   Storage Encryption: ${config.storageEncryption ? 'Enabled' : 'Disabled'}`);
    console.log(`   Max Concurrent Requests: ${config.maxConcurrentRequests}`);
    console.log(`   Circuit Breaker Threshold: ${config.circuitBreakerThreshold}`);
  });
  console.log();

  // 2. Initialize Production Client
  console.log('🚀 Initializing Production Client...');
  
  const client = new ProductionICNClient('development'); // Use development for demo
  
  try {
    await client.initialize();
    console.log('✅ Production client initialized successfully');
  } catch (error) {
    console.log('❌ Failed to initialize client:', ErrorUtils.getErrorMessage(error));
    console.log('💡 This is expected in demo environment without running ICN nodes');
  }
  console.log();

  // 3. Connection Status and Health Check
  console.log('🔍 Connection Status and Health Check...');
  
  const connectionStatus = client.getConnectionStatus();
  console.log('📊 Connection Pool Status:');
  console.log(`   Total Endpoints: ${connectionStatus.total || 0}`);
  console.log(`   Active Connections: ${connectionStatus.active || 0}`);
  console.log(`   Failed Connections: ${connectionStatus.failed || 0}`);
  
  if (connectionStatus.activeEndpoints) {
    console.log('   Active Endpoints:');
    connectionStatus.activeEndpoints.forEach((endpoint: string, index: number) => {
      console.log(`     ${index + 1}. ${endpoint}`);
    });
  }
  
  if (connectionStatus.failedEndpoints) {
    console.log('   Failed Endpoints:');
    connectionStatus.failedEndpoints.forEach((endpoint: string, index: number) => {
      console.log(`     ${index + 1}. ${endpoint}`);
    });
  }
  
  const healthCheck = await client.performHealthCheck();
  console.log('\n🏥 Health Check Results:');
  console.log(`   Status: ${healthCheck.status.toUpperCase()}`);
  console.log(`   Timestamp: ${healthCheck.timestamp}`);
  console.log();

  // 4. Metrics and Monitoring
  console.log('📊 Metrics and Monitoring...');
  
  const metrics = client.getMetrics();
  console.log('📈 Current Metrics:');
  Object.entries(metrics).forEach(([metric, value]) => {
    console.log(`   ${metric}: ${value}`);
  });
  console.log();

  // 5. Security Configuration Examples
  console.log('🔒 Security Configuration Examples...');
  
  console.log('🔐 Storage Encryption Configuration:');
  console.log('```typescript');
  console.log('const secureStorage = createSecureStorage("@myapp:", {');
  console.log('  enableEncryption: true,');
  console.log('  passphrase: process.env.STORAGE_PASSPHRASE');
  console.log('});');
  console.log('```');
  
  console.log('\n🔑 Authentication Configuration:');
  console.log('```typescript');
  console.log('const client = new ICNClient({');
  console.log('  nodeEndpoint: process.env.ICN_NODE_ENDPOINT,');
  console.log('  privateKey: process.env.ICN_PRIVATE_KEY,');
  console.log('  network: process.env.ICN_NETWORK,');
  console.log('  timeout: parseInt(process.env.ICN_TIMEOUT || "30000"),');
  console.log('  storage: secureStorage');
  console.log('});');
  console.log('```');
  
  console.log('\n🌐 TLS/HTTPS Configuration:');
  console.log('• Always use HTTPS endpoints in production');
  console.log('• Validate SSL certificates');
  console.log('• Use certificate pinning for critical applications');
  console.log('• Implement proper CORS policies');
  console.log();

  // 6. Performance Optimization
  console.log('⚡ Performance Optimization Examples...');
  
  console.log('🔄 Connection Pooling Benefits:');
  console.log('• Reduced connection overhead');
  console.log('• Load balancing across multiple nodes');
  console.log('• Automatic failover and recovery');
  console.log('• Health monitoring and circuit breaking');
  
  console.log('\n📦 Request Batching Example:');
  console.log('```typescript');
  console.log('// Batch multiple operations');
  console.log('const results = await Promise.allSettled([');
  console.log('  client.getTrustScore("did:key:user1"),');
  console.log('  client.getTrustScore("did:key:user2"),');
  console.log('  client.getTrustScore("did:key:user3")');
  console.log(']);');
  console.log('```');
  
  console.log('\n💾 Caching Strategy:');
  console.log('```typescript');
  console.log('const cache = new Map();');
  console.log('');
  console.log('async function getCachedTrustScore(did: string) {');
  console.log('  const cacheKey = `trust_score_${did}`;');
  console.log('  const cached = cache.get(cacheKey);');
  console.log('  ');
  console.log('  if (cached && Date.now() - cached.timestamp < 300000) {');
  console.log('    return cached.data; // 5 minute cache');
  console.log('  }');
  console.log('  ');
  console.log('  const score = await client.getTrustScore(did);');
  console.log('  cache.set(cacheKey, { data: score, timestamp: Date.now() });');
  console.log('  return score;');
  console.log('}');
  console.log('```');
  console.log();

  // 7. Error Handling and Monitoring
  console.log('🛡️  Production Error Handling...');
  
  console.log('📊 Error Tracking Integration:');
  console.log('```typescript');
  console.log('import * as Sentry from "@sentry/node";');
  console.log('');
  console.log('try {');
  console.log('  await client.submitProposal(proposal);');
  console.log('} catch (error) {');
  console.log('  // Log to monitoring service');
  console.log('  Sentry.captureException(error, {');
  console.log('    tags: { operation: "governance.submitProposal" },');
  console.log('    extra: { proposal }');
  console.log('  });');
  console.log('  ');
  console.log('  // Handle gracefully');
  console.log('  if (ErrorUtils.isRetryableError(error)) {');
  console.log('    // Queue for retry');
  console.log('  } else {');
  console.log('    // Show user error');
  console.log('  }');
  console.log('}');
  console.log('```');
  
  console.log('\n📈 Metrics Collection:');
  console.log('```typescript');
  console.log('import { createPrometheusMetrics } from "./metrics";');
  console.log('');
  console.log('const metrics = createPrometheusMetrics();');
  console.log('');
  console.log('// Custom metric middleware');
  console.log('async function withMetrics(operation: string, fn: Function) {');
  console.log('  const timer = metrics.startTimer("icn_operation_duration", { operation });');
  console.log('  metrics.increment("icn_operation_total", { operation });');
  console.log('  ');
  console.log('  try {');
  console.log('    const result = await fn();');
  console.log('    metrics.increment("icn_operation_success", { operation });');
  console.log('    return result;');
  console.log('  } catch (error) {');
  console.log('    metrics.increment("icn_operation_error", { operation, error: error.constructor.name });');
  console.log('    throw error;');
  console.log('  } finally {');
  console.log('    timer();');
  console.log('  }');
  console.log('}');
  console.log('```');
  console.log();

  // 8. Deployment Considerations
  console.log('🚀 Deployment Considerations...');
  
  console.log('🐳 Docker Configuration:');
  console.log('```dockerfile');
  console.log('FROM node:18-alpine');
  console.log('');
  console.log('# Security updates');
  console.log('RUN apk update && apk upgrade');
  console.log('');
  console.log('# Non-root user');
  console.log('RUN addgroup -g 1001 -S nodejs');
  console.log('RUN adduser -S icnapp -u 1001');
  console.log('');
  console.log('WORKDIR /app');
  console.log('COPY package*.json ./');
  console.log('RUN npm ci --only=production');
  console.log('');
  console.log('COPY . .');
  console.log('USER icnapp');
  console.log('');
  console.log('# Health check');
  console.log('HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \\');
  console.log('  CMD node healthcheck.js');
  console.log('');
  console.log('CMD ["node", "index.js"]');
  console.log('```');
  
  console.log('\n☸️  Kubernetes Configuration:');
  console.log('```yaml');
  console.log('apiVersion: apps/v1');
  console.log('kind: Deployment');
  console.log('metadata:');
  console.log('  name: icn-app');
  console.log('spec:');
  console.log('  replicas: 3');
  console.log('  selector:');
  console.log('    matchLabels:');
  console.log('      app: icn-app');
  console.log('  template:');
  console.log('    metadata:');
  console.log('      labels:');
  console.log('        app: icn-app');
  console.log('    spec:');
  console.log('      containers:');
  console.log('      - name: icn-app');
  console.log('        image: myregistry/icn-app:latest');
  console.log('        ports:');
  console.log('        - containerPort: 3000');
  console.log('        env:');
  console.log('        - name: ICN_NODE_ENDPOINT');
  console.log('          valueFrom:');
  console.log('            secretKeyRef:');
  console.log('              name: icn-secrets');
  console.log('              key: node-endpoint');
  console.log('        livenessProbe:');
  console.log('          httpGet:');
  console.log('            path: /health');
  console.log('            port: 3000');
  console.log('        readinessProbe:');
  console.log('          httpGet:');
  console.log('            path: /ready');
  console.log('            port: 3000');
  console.log('```');
  console.log();

  // 9. Environment Variables
  console.log('🌍 Environment Variables Configuration...');
  
  console.log('📋 Required Environment Variables:');
  const envVars = [
    { name: 'ICN_NODE_ENDPOINT', description: 'Primary ICN node endpoint URL', required: true },
    { name: 'ICN_NODE_ENDPOINTS', description: 'Comma-separated list of backup endpoints', required: false },
    { name: 'ICN_NETWORK', description: 'Network type (mainnet, testnet, devnet)', required: true },
    { name: 'ICN_PRIVATE_KEY', description: 'Private key for authentication', required: true },
    { name: 'ICN_STORAGE_PASSPHRASE', description: 'Passphrase for storage encryption', required: true },
    { name: 'ICN_TIMEOUT', description: 'Request timeout in milliseconds', required: false },
    { name: 'ICN_LOG_LEVEL', description: 'Logging level (debug, info, warn, error)', required: false },
    { name: 'ICN_METRICS_ENABLED', description: 'Enable metrics collection (true/false)', required: false },
    { name: 'ICN_CIRCUIT_BREAKER_THRESHOLD', description: 'Circuit breaker failure threshold', required: false }
  ];
  
  envVars.forEach(envVar => {
    const status = envVar.required ? '🔴 Required' : '🟡 Optional';
    console.log(`   ${envVar.name}: ${envVar.description} [${status}]`);
  });
  
  console.log('\n📄 Example .env file:');
  console.log('```env');
  console.log('ICN_NODE_ENDPOINT=https://node-1.icn.network');
  console.log('ICN_NODE_ENDPOINTS=https://node-2.icn.network,https://node-3.icn.network');
  console.log('ICN_NETWORK=mainnet');
  console.log('ICN_PRIVATE_KEY=your-private-key-here');
  console.log('ICN_STORAGE_PASSPHRASE=your-secure-passphrase');
  console.log('ICN_TIMEOUT=30000');
  console.log('ICN_LOG_LEVEL=info');
  console.log('ICN_METRICS_ENABLED=true');
  console.log('ICN_CIRCUIT_BREAKER_THRESHOLD=20');
  console.log('```');
  console.log();

  // 10. Best Practices Summary
  console.log('💡 Production Best Practices Summary...');
  
  console.log('🔧 Configuration:');
  console.log('• Use environment-specific configurations');
  console.log('• Enable storage encryption in production');
  console.log('• Configure appropriate timeouts and retry limits');
  console.log('• Use connection pooling for high availability');
  console.log('• Implement circuit breaker patterns');
  
  console.log('\n🔒 Security:');
  console.log('• Store private keys and passphrases securely');
  console.log('• Use HTTPS endpoints with certificate validation');
  console.log('• Implement proper authentication and authorization');
  console.log('• Regular security audits and dependency updates');
  console.log('• Follow principle of least privilege');
  
  console.log('\n📊 Monitoring:');
  console.log('• Collect comprehensive metrics and logs');
  console.log('• Set up alerting for critical thresholds');
  console.log('• Monitor connection pool health');
  console.log('• Track error rates and response times');
  console.log('• Implement distributed tracing');
  
  console.log('\n⚡ Performance:');
  console.log('• Use connection pooling and load balancing');
  console.log('• Implement caching for frequently accessed data');
  console.log('• Batch requests when possible');
  console.log('• Optimize resource usage and memory management');
  console.log('• Regular performance testing and optimization');
  
  console.log('\n🛡️  Reliability:');
  console.log('• Implement graceful error handling');
  console.log('• Use retry logic with exponential backoff');
  console.log('• Provide fallback mechanisms');
  console.log('• Regular health checks and monitoring');
  console.log('• Disaster recovery and backup strategies');

  // Cleanup
  try {
    await client.shutdown();
    console.log('\n🎉 Production Configuration example completed successfully!');
  } catch (error) {
    console.log('\n⚠️  Shutdown completed with minor issues (expected in demo)');
  }

  console.log('\n🏭 Production Readiness Checklist:');
  console.log('• ✅ Environment-specific configuration implemented');
  console.log('• ✅ Security best practices documented');
  console.log('• ✅ Error handling and monitoring setup');
  console.log('• ✅ Performance optimization strategies');
  console.log('• ✅ Deployment configurations provided');
  console.log('• ✅ Comprehensive logging and metrics');
  console.log('• ✅ Health checks and status endpoints');
  console.log('• ✅ Documentation and best practices guide');
}

// Run the example
if (require.main === module) {
  productionConfigExample().catch(console.error);
}

export { productionConfigExample, ProductionICNClient, environments };