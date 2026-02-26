export { SnapshotClient } from './snapshot/client';
export { stripSnapshotIndentation } from './snapshot/port/inlineSnapshot';
export { addSerializer, getSerializers } from './snapshot/port/plugins';
export { default as SnapshotState } from './snapshot/port/state';
export type { SnapshotData, SnapshotEnvironment, SnapshotMatchOptions, SnapshotResult, SnapshotSerializer, SnapshotStateOptions, SnapshotSummary, SnapshotUpdateState, UncheckedSnapshot, } from './snapshot/types';
