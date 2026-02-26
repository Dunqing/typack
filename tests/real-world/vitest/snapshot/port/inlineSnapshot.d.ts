import type MagicString from 'magic-string';
import type { SnapshotEnvironment } from '../types';
export interface InlineSnapshot {
    snapshot: string;
    testId: string;
    file: string;
    line: number;
    column: number;
}
export declare function saveInlineSnapshots(environment: SnapshotEnvironment, snapshots: Array<InlineSnapshot>): Promise<void>;
export declare function replaceInlineSnap(code: string, s: MagicString, currentIndex: number, newSnap: string): boolean;
export declare function stripSnapshotIndentation(inlineSnapshot: string): string;
