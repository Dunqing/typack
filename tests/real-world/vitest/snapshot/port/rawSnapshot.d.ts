import type { SnapshotEnvironment } from '../types';
export interface RawSnapshotInfo {
    file: string;
    readonly?: boolean;
    content?: string;
}
export interface RawSnapshot extends RawSnapshotInfo {
    snapshot: string;
    file: string;
}
export declare function saveRawSnapshots(environment: SnapshotEnvironment, snapshots: Array<RawSnapshot>): Promise<void>;
