export type {
  EventEmitterOptions,
  EventListener,
  TypedEventMap,
  Disposable,
  AsyncDisposable,
  Result,
  MaybePromise,
  Logger,
  RetryOptions,
} from "./types";
export { DEFAULT_RETRY_OPTIONS, createLogger, delay, clamp } from "./types";

export type {
  Collection,
  OrderedCollection,
  ReadonlyCollection,
  MapLike,
  Predicate,
  Mapper,
  Reducer,
  QueryableCollection,
  TreeNode,
  Graph,
  PriorityQueue,
} from "./collections";
export { createStack } from "./collections";

export interface AppConfig {
  name: string;
  version: string;
  environment: "development" | "staging" | "production";
  debug: boolean;
  features: ReadonlyMap<string, boolean>;
}

export interface Plugin<T = unknown> {
  readonly name: string;
  readonly version: string;
  install(app: T): void;
  uninstall?(app: T): void;
}

export interface PluginManager<T> {
  register(plugin: Plugin<T>): void;
  unregister(name: string): boolean;
  get(name: string): Plugin<T> | undefined;
  list(): readonly Plugin<T>[];
}

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

export type Awaited<T> = T extends Promise<infer U> ? Awaited<U> : T;

export function createApp(config: AppConfig): {
  readonly config: DeepReadonly<AppConfig>;
  start(): Promise<void>;
  stop(): Promise<void>;
} {
  return {
    config: config as DeepReadonly<AppConfig>,
    async start(): Promise<void> {},
    async stop(): Promise<void> {},
  };
}
