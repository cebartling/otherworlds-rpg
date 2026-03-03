/**
 * Typed error classes for the Otherworlds web client.
 *
 * These are client-importable errors (NOT in $lib/server/) that map
 * to the backend's DomainError variants. They can be used in both
 * server and client code for type-safe error handling.
 */

export class NotFoundError extends Error {
  constructor(message = 'The requested resource was not found') {
    super(message);
    this.name = 'NotFoundError';
  }
}

export class ValidationError extends Error {
  constructor(message = 'Invalid input') {
    super(message);
    this.name = 'ValidationError';
  }
}

export class ConcurrencyConflictError extends Error {
  constructor(message = 'The resource was modified by another request') {
    super(message);
    this.name = 'ConcurrencyConflictError';
  }
}

export class InfrastructureError extends Error {
  constructor(message = 'An internal error occurred') {
    super(message);
    this.name = 'InfrastructureError';
  }
}
