/**
 * Enhanced error handling for ICN TypeScript SDK
 */

// Base ICN error class
export class ICNError extends Error {
  public readonly code: string;
  public readonly details?: any;
  public readonly correlationId?: string;

  constructor(
    code: string,
    message: string,
    details?: any,
    correlationId?: string
  ) {
    super(message);
    this.name = 'ICNError';
    this.code = code;
    this.details = details;
    this.correlationId = correlationId;
  }

  toJSON() {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      details: this.details,
      correlationId: this.correlationId,
      stack: this.stack,
    };
  }
}

// Connection-related errors
export class ICNConnectionError extends ICNError {
  constructor(message: string, details?: any) {
    super('CONNECTION_ERROR', message, details);
    this.name = 'ICNConnectionError';
  }
}

// Authentication/authorization errors
export class ICNAuthError extends ICNError {
  constructor(message: string, details?: any) {
    super('AUTH_ERROR', message, details);
    this.name = 'ICNAuthError';
  }
}

// Validation errors
export class ICNValidationError extends ICNError {
  public readonly field?: string;

  constructor(message: string, field?: string, details?: any) {
    super('VALIDATION_ERROR', message, details);
    this.name = 'ICNValidationError';
    this.field = field;
  }
}

// Network/API errors
export class ICNNetworkError extends ICNError {
  public readonly statusCode?: number;
  public readonly url?: string;

  constructor(
    message: string,
    statusCode?: number,
    url?: string,
    details?: any
  ) {
    super('NETWORK_ERROR', message, details);
    this.name = 'ICNNetworkError';
    this.statusCode = statusCode;
    this.url = url;
  }
}

// Governance-specific errors
export class ICNGovernanceError extends ICNError {
  constructor(message: string, details?: any) {
    super('GOVERNANCE_ERROR', message, details);
    this.name = 'ICNGovernanceError';
  }
}

// Credential-specific errors
export class ICNCredentialError extends ICNError {
  constructor(message: string, details?: any) {
    super('CREDENTIAL_ERROR', message, details);
    this.name = 'ICNCredentialError';
  }
}

// Trust-specific errors
export class ICNTrustError extends ICNError {
  constructor(message: string, details?: any) {
    super('TRUST_ERROR', message, details);
    this.name = 'ICNTrustError';
  }
}

// Mesh/Job-specific errors
export class ICNMeshError extends ICNError {
  constructor(message: string, details?: any) {
    super('MESH_ERROR', message, details);
    this.name = 'ICNMeshError';
  }
}

// Storage-specific errors
export class ICNStorageError extends ICNError {
  constructor(message: string, details?: any) {
    super('STORAGE_ERROR', message, details);
    this.name = 'ICNStorageError';
  }
}

// Token-specific errors
export class ICNTokenError extends ICNError {
  constructor(message: string, details?: any) {
    super('TOKEN_ERROR', message, details);
    this.name = 'ICNTokenError';
  }
}

// Timeout errors
export class ICNTimeoutError extends ICNError {
  public readonly timeoutMs: number;

  constructor(message: string, timeoutMs: number, details?: any) {
    super('TIMEOUT_ERROR', message, details);
    this.name = 'ICNTimeoutError';
    this.timeoutMs = timeoutMs;
  }
}

// Rate limiting errors
export class ICNRateLimitError extends ICNError {
  public readonly retryAfter?: number;

  constructor(message: string, retryAfter?: number, details?: any) {
    super('RATE_LIMIT_ERROR', message, details);
    this.name = 'ICNRateLimitError';
    this.retryAfter = retryAfter;
  }
}

/**
 * Error factory to create appropriate error types based on error data
 */
export class ErrorFactory {
  static fromApiError(
    status: number,
    message: string,
    details?: any,
    url?: string
  ): ICNError {
    switch (status) {
      case 400:
        return new ICNValidationError(message, undefined, details);
      case 401:
      case 403:
        return new ICNAuthError(message, details);
      case 408:
        return new ICNTimeoutError(message, details?.timeout || 30000, details);
      case 429:
        return new ICNRateLimitError(
          message,
          details?.retryAfter,
          details
        );
      case 500:
      case 502:
      case 503:
      case 504:
        return new ICNNetworkError(message, status, url, details);
      default:
        if (status >= 400 && status < 500) {
          return new ICNValidationError(message, undefined, details);
        } else if (status >= 500) {
          return new ICNNetworkError(message, status, url, details);
        } else {
          return new ICNError('UNKNOWN_ERROR', message, details);
        }
    }
  }

  static fromErrorType(type: string, message: string, details?: any): ICNError {
    switch (type.toLowerCase()) {
      case 'connection':
      case 'connection_error':
        return new ICNConnectionError(message, details);
      case 'auth':
      case 'auth_error':
      case 'authentication':
      case 'authorization':
        return new ICNAuthError(message, details);
      case 'validation':
      case 'validation_error':
        return new ICNValidationError(message, undefined, details);
      case 'governance':
      case 'governance_error':
        return new ICNGovernanceError(message, details);
      case 'credential':
      case 'credential_error':
        return new ICNCredentialError(message, details);
      case 'trust':
      case 'trust_error':
        return new ICNTrustError(message, details);
      case 'mesh':
      case 'mesh_error':
      case 'job':
      case 'job_error':
        return new ICNMeshError(message, details);
      case 'storage':
      case 'storage_error':
        return new ICNStorageError(message, details);
      case 'token':
      case 'token_error':
        return new ICNTokenError(message, details);
      case 'timeout':
      case 'timeout_error':
        return new ICNTimeoutError(message, details?.timeout || 30000, details);
      case 'rate_limit':
      case 'rate_limit_error':
        return new ICNRateLimitError(message, details?.retryAfter, details);
      case 'network':
      case 'network_error':
        return new ICNNetworkError(message, undefined, undefined, details);
      default:
        return new ICNError(type.toUpperCase(), message, details);
    }
  }

  static fromUnknownError(error: unknown): ICNError {
    if (error instanceof ICNError) {
      return error;
    }

    if (error instanceof Error) {
      // Try to detect error type from message
      const message = error.message.toLowerCase();
      if (message.includes('network') || message.includes('fetch')) {
        return new ICNNetworkError(error.message);
      }
      if (message.includes('timeout')) {
        return new ICNTimeoutError(error.message, 30000);
      }
      if (message.includes('auth') || message.includes('unauthorized')) {
        return new ICNAuthError(error.message);
      }
      if (message.includes('validation') || message.includes('invalid')) {
        return new ICNValidationError(error.message);
      }

      return new ICNError('UNKNOWN_ERROR', error.message, { originalError: error });
    }

    if (typeof error === 'string') {
      return new ICNError('UNKNOWN_ERROR', error);
    }

    return new ICNError('UNKNOWN_ERROR', 'An unknown error occurred', { originalError: error });
  }
}

/**
 * Utility functions for error handling
 */
export const ErrorUtils = {
  /**
   * Check if an error is of a specific type
   */
  isErrorType<T extends ICNError>(error: unknown, ErrorClass: new (...args: any[]) => T): error is T {
    return error instanceof ErrorClass;
  },

  /**
   * Extract error message from various error types
   */
  getErrorMessage(error: unknown): string {
    if (error instanceof ICNError) {
      return error.message;
    }
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === 'string') {
      return error;
    }
    return 'An unknown error occurred';
  },

  /**
   * Check if an error is retryable
   */
  isRetryableError(error: unknown): boolean {
    if (error instanceof ICNNetworkError) {
      // Retry on 5xx errors, but not 4xx
      return !error.statusCode || error.statusCode >= 500;
    }
    if (error instanceof ICNTimeoutError) {
      return true;
    }
    if (error instanceof ICNConnectionError) {
      return true;
    }
    return false;
  },

  /**
   * Get suggested retry delay for retryable errors
   */
  getRetryDelay(error: unknown, attempt: number): number {
    if (error instanceof ICNRateLimitError && error.retryAfter) {
      return error.retryAfter * 1000; // Convert to milliseconds
    }
    
    // Exponential backoff: 1s, 2s, 4s, 8s, max 30s
    return Math.min(1000 * Math.pow(2, attempt - 1), 30000);
  },

  /**
   * Wrap a function call with error conversion
   */
  async wrapWithErrorHandling<T>(
    fn: () => Promise<T>,
    errorContext?: string
  ): Promise<T> {
    try {
      return await fn();
    } catch (error) {
      const icnError = ErrorFactory.fromUnknownError(error);
      if (errorContext) {
        icnError.details = {
          ...icnError.details,
          context: errorContext,
        };
      }
      throw icnError;
    }
  },
};