import { PrettyFormatOptions } from "@vitest/pretty-format";

//#region tests/real-world/vitest/utils/display.d.ts
type Inspect = (value: unknown, options: Options) => string;
interface Options {
  showHidden: boolean;
  depth: number;
  colors: boolean;
  customInspect: boolean;
  showProxy: boolean;
  maxArrayLength: number;
  breakLength: number;
  truncate: number;
  seen: unknown[];
  inspect: Inspect;
  stylize: (value: string, styleType: string) => string;
}
type LoupeOptions = Partial<Options>;
interface StringifyOptions extends PrettyFormatOptions {
  maxLength?: number;
  filterNode?: string | ((node: any) => boolean);
}
//#endregion
//#region tests/real-world/vitest/utils/types.d.ts
type Awaitable<T> = T | PromiseLike<T>;
type Nullable<T> = T | null | undefined;
type Arrayable<T> = T | Array<T>;
type ArgumentsType<T> = T extends (...args: infer U) => any ? U : never;
type MergeInsertions<T> = T extends object ? { [K in keyof T] : MergeInsertions<T[K]> } : T;
type DeepMerge<
  F,
  S
> = MergeInsertions<{ [K in keyof F | keyof S] : K extends keyof S & keyof F ? DeepMerge<F[K], S[K]> : K extends keyof S ? S[K] : K extends keyof F ? F[K] : never }>;
interface Constructable {
  new (...args: any[]): any;
}
interface ParsedStack {
  method: string;
  file: string;
  line: number;
  column: number;
}
interface SerializedError {
  message: string;
  stacks?: ParsedStack[];
  stack?: string;
  name?: string;
  cause?: SerializedError;
  [key: string]: unknown;
}
interface TestError extends SerializedError {
  cause?: TestError;
  diff?: string;
  actual?: string;
  expected?: string;
}
//#endregion
//#region tests/real-world/vitest/utils/helpers.d.ts
type DeferPromise<T> = Promise<T> & {
  resolve: (value: T | PromiseLike<T>) => void;
  reject: (reason?: any) => void;
};
//#endregion
//#region tests/real-world/vitest/utils/timers.d.ts
interface SafeTimers {
  nextTick?: (cb: () => void) => void;
  setImmediate?: {
    <TArgs extends any[]>(callback: (...args: TArgs) => void, ...args: TArgs): any;
    __promisify__: <T = void>(value?: T, options?: any) => Promise<T>;
  };
  clearImmediate?: (immediateId: any) => void;
  setTimeout: typeof setTimeout;
  setInterval: typeof setInterval;
  clearInterval: typeof clearInterval;
  clearTimeout: typeof clearTimeout;
  queueMicrotask: typeof queueMicrotask;
}
//#endregion
export { type ArgumentsType, type Arrayable, type Awaitable, type Constructable, type DeepMerge, type DeferPromise, type LoupeOptions, type MergeInsertions, type Nullable, type ParsedStack, type SafeTimers, type SerializedError, type StringifyOptions, type TestError };