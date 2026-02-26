import { type ComponentInternalInstance } from './component';
export declare enum SchedulerJobFlags {
    QUEUED = 1,
    PRE = 2,
    /**
     * Indicates whether the effect is allowed to recursively trigger itself
     * when managed by the scheduler.
     *
     * By default, a job cannot trigger itself because some built-in method calls,
     * e.g. Array.prototype.push actually performs reads as well (#1740) which
     * can lead to confusing infinite loops.
     * The allowed cases are component update functions and watch callbacks.
     * Component update functions may update child component props, which in turn
     * trigger flush: "pre" watch callbacks that mutates state that the parent
     * relies on (#1801). Watch callbacks doesn't track its dependencies so if it
     * triggers itself again, it's likely intentional and it is the user's
     * responsibility to perform recursive state mutation that eventually
     * stabilizes (#1727).
     */
    ALLOW_RECURSE = 4,
    DISPOSED = 8
}
export interface SchedulerJob extends Function {
    id?: number;
    /**
     * flags can technically be undefined, but it can still be used in bitwise
     * operations just like 0.
     */
    flags?: SchedulerJobFlags;
    /**
     * Attached by renderer.ts when setting up a component's render effect
     * Used to obtain component information when reporting max recursive updates.
     */
    i?: ComponentInternalInstance;
}
export type SchedulerJobs = SchedulerJob | SchedulerJob[];
type CountMap = Map<SchedulerJob, number>;
export declare function nextTick(): Promise<void>;
export declare function nextTick<T, R>(this: T, fn: (this: T) => R | Promise<R>): Promise<R>;
export declare function queueJob(job: SchedulerJob): void;
export declare function queuePostFlushCb(cb: SchedulerJobs): void;
export declare function flushPreFlushCbs(instance?: ComponentInternalInstance, seen?: CountMap, i?: number): void;
export declare function flushPostFlushCbs(seen?: CountMap): void;
export {};
