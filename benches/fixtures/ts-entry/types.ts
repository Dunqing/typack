export interface EventEmitterOptions {
  maxListeners: number;
  captureRejections: boolean;
}

export type EventListener<T = unknown> = (event: T) => void;

export interface TypedEventMap {
  connect: { host: string; port: number };
  disconnect: { reason: string; code: number };
  error: { message: string; stack?: string };
  data: { payload: Uint8Array; timestamp: number };
  message: { id: string; body: string; metadata: Record<string, string> };
}

export interface Disposable {
  dispose(): void;
}

export interface AsyncDisposable {
  asyncDispose(): Promise<void>;
}

export type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

export type MaybePromise<T> = T | Promise<T>;

export interface Logger {
  debug(message: string, ...args: unknown[]): void;
  info(message: string, ...args: unknown[]): void;
  warn(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
}

export interface RetryOptions {
  maxRetries: number;
  initialDelay: number;
  maxDelay: number;
  backoffFactor: number;
  retryableErrors?: ReadonlyArray<string>;
}

export const DEFAULT_RETRY_OPTIONS: RetryOptions = {
  maxRetries: 3,
  initialDelay: 100,
  maxDelay: 5000,
  backoffFactor: 2,
};

export function createLogger(prefix: string): Logger {
  return {
    debug: (msg: string, ...args: unknown[]) => console.debug(`[${prefix}] ${msg}`, ...args),
    info: (msg: string, ...args: unknown[]) => console.info(`[${prefix}] ${msg}`, ...args),
    warn: (msg: string, ...args: unknown[]) => console.warn(`[${prefix}] ${msg}`, ...args),
    error: (msg: string, ...args: unknown[]) => console.error(`[${prefix}] ${msg}`, ...args),
  };
}

export function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
