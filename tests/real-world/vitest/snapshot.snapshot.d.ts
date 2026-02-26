import { OptionsReceived as PrettyFormatOptions, Plugin as PrettyFormatPlugin, Plugins as PrettyFormatPlugins } from "@vitest/pretty-format";
import { ParsedStack } from "@vitest/utils";

//#region tests/real-world/vitest/snapshot/types/environment.d.ts
interface SnapshotEnvironment {
  getVersion: () => string;
  getHeader: () => string;
  resolvePath: (filepath: string) => Promise<string>;
  resolveRawPath: (testPath: string, rawPath: string) => Promise<string>;
  saveSnapshotFile: (filepath: string, snapshot: string) => Promise<void>;
  readSnapshotFile: (filepath: string) => Promise<string | null>;
  removeSnapshotFile: (filepath: string) => Promise<void>;
  processStackTrace?: (stack: ParsedStack) => ParsedStack;
}
//#endregion
//#region tests/real-world/vitest/snapshot/types/index.d.ts
type SnapshotData = Record<string, string>;
type SnapshotUpdateState = "all" | "new" | "none";
type SnapshotSerializer = PrettyFormatPlugin;
interface SnapshotStateOptions {
  updateSnapshot: SnapshotUpdateState;
  snapshotEnvironment: SnapshotEnvironment;
  expand?: boolean;
  snapshotFormat?: PrettyFormatOptions;
  resolveSnapshotPath?: (path: string, extension: string, context?: any) => string;
}
interface SnapshotMatchOptions {
  testId: string;
  testName: string;
  received: unknown;
  key?: string;
  inlineSnapshot?: string;
  isInline: boolean;
  error?: Error;
  rawSnapshot?: RawSnapshotInfo;
}
interface SnapshotResult {
  filepath: string;
  added: number;
  fileDeleted: boolean;
  matched: number;
  unchecked: number;
  uncheckedKeys: Array<string>;
  unmatched: number;
  updated: number;
}
interface UncheckedSnapshot {
  filePath: string;
  keys: Array<string>;
}
interface SnapshotSummary {
  added: number;
  didUpdate: boolean;
  failure: boolean;
  filesAdded: number;
  filesRemoved: number;
  filesRemovedList: Array<string>;
  filesUnmatched: number;
  filesUpdated: number;
  matched: number;
  total: number;
  unchecked: number;
  uncheckedKeysByFile: Array<UncheckedSnapshot>;
  unmatched: number;
  updated: number;
}
//#endregion
//#region tests/real-world/vitest/snapshot/port/rawSnapshot.d.ts
interface RawSnapshotInfo {
  file: string;
  readonly?: boolean;
  content?: string;
}
//#endregion
//#region tests/real-world/vitest/snapshot/port/utils.d.ts
declare class DefaultMap<
  K,
  V
> extends Map<K, V> {
  private defaultFn;
  constructor(defaultFn: (key: K) => V, entries?: Iterable<readonly [K, V]>);
  get(key: K): V;
}
declare class CounterMap<K> extends DefaultMap<K, number> {
  constructor();
  _total: number | undefined;
  valueOf(): number;
  increment(key: K): void;
  total(): number;
}
//#endregion
//#region tests/real-world/vitest/snapshot/port/state.d.ts
interface SnapshotReturnOptions {
  actual: string;
  count: number;
  expected?: string;
  key: string;
  pass: boolean;
}
interface SaveStatus {
  deleted: boolean;
  saved: boolean;
}
declare class SnapshotState {
  testFilePath: string;
  snapshotPath: string;
  private _counters;
  private _dirty;
  private _updateSnapshot;
  private _snapshotData;
  private _initialData;
  private _inlineSnapshots;
  private _inlineSnapshotStacks;
  private _testIdToKeys;
  private _rawSnapshots;
  private _uncheckedKeys;
  private _snapshotFormat;
  private _environment;
  private _fileExists;
  expand: boolean;
  private _added;
  private _matched;
  private _unmatched;
  private _updated;
  get added(): CounterMap<string>;
  set added(value: number);
  get matched(): CounterMap<string>;
  set matched(value: number);
  get unmatched(): CounterMap<string>;
  set unmatched(value: number);
  get updated(): CounterMap<string>;
  set updated(value: number);
  private constructor();
  static create(testFilePath: string, options: SnapshotStateOptions): Promise<SnapshotState>;
  get environment(): SnapshotEnvironment;
  markSnapshotsAsCheckedForTest(testName: string): void;
  clearTest(testId: string): void;
  protected _inferInlineSnapshotStack(stacks: ParsedStack[]): ParsedStack | null;
  private _addSnapshot;
  save(): Promise<SaveStatus>;
  getUncheckedCount(): number;
  getUncheckedKeys(): Array<string>;
  removeUncheckedKeys(): void;
  match({ testId, testName, received, key, inlineSnapshot, isInline, error, rawSnapshot }: SnapshotMatchOptions): SnapshotReturnOptions;
  pack(): Promise<SnapshotResult>;
}
//#endregion
//#region tests/real-world/vitest/snapshot/client.d.ts
interface AssertOptions {
  received: unknown;
  filepath: string;
  name: string;
  /**
  * Not required but needed for `SnapshotClient.clearTest` to implement test-retry behavior.
  * @default name
  */
  testId?: string;
  message?: string;
  isInline?: boolean;
  properties?: object;
  inlineSnapshot?: string;
  error?: Error;
  errorMessage?: string;
  rawSnapshot?: RawSnapshotInfo;
}
interface SnapshotClientOptions {
  isEqual?: (received: unknown, expected: unknown) => boolean;
}
declare class SnapshotClient {
  private options;
  snapshotStateMap: Map<string, SnapshotState>;
  constructor(options?: SnapshotClientOptions);
  setup(filepath: string, options: SnapshotStateOptions): Promise<void>;
  finish(filepath: string): Promise<SnapshotResult>;
  skipTest(filepath: string, testName: string): void;
  clearTest(filepath: string, testId: string): void;
  getSnapshotState(filepath: string): SnapshotState;
  assert(options: AssertOptions): void;
  assertRaw(options: AssertOptions): Promise<void>;
  clear(): void;
}
//#endregion
//#region tests/real-world/vitest/snapshot/port/inlineSnapshot.d.ts
declare function stripSnapshotIndentation(inlineSnapshot: string): string;
//#endregion
//#region tests/real-world/vitest/snapshot/port/plugins.d.ts
declare function addSerializer(plugin: PrettyFormatPlugin): void;
declare function getSerializers(): PrettyFormatPlugins;
//#endregion
export { SnapshotClient, type SnapshotData, type SnapshotEnvironment, type SnapshotMatchOptions, type SnapshotResult, type SnapshotSerializer, SnapshotState, type SnapshotStateOptions, type SnapshotSummary, type SnapshotUpdateState, type UncheckedSnapshot, addSerializer, getSerializers, stripSnapshotIndentation };