import { type ComponentInternalInstance } from '../component';
export declare function useId(): string;
/**
 * There are 3 types of async boundaries:
 * - async components
 * - components with async setup()
 * - components with serverPrefetch
 */
export declare function markAsyncBoundary(instance: ComponentInternalInstance): void;
