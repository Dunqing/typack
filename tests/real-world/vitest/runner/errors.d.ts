import type { CancelReason } from './types/runner';
import type { TaskBase } from './types/tasks';
export declare class PendingError extends Error {
    message: string;
    note: string | undefined;
    code: string;
    taskId: string;
    constructor(message: string, task: TaskBase, note: string | undefined);
}
export declare class TestRunAbortError extends Error {
    name: string;
    reason: CancelReason;
    constructor(message: string, reason: CancelReason);
}
export declare class FixtureDependencyError extends Error {
    name: string;
}
export declare class AroundHookSetupError extends Error {
    name: string;
}
export declare class AroundHookTeardownError extends Error {
    name: string;
}
export declare class AroundHookMultipleCallsError extends Error {
    name: string;
}
