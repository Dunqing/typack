import type { ComponentInternalInstance } from './component';
import { LifecycleHooks } from './enums';
import { WatchErrorCodes } from '@vue/reactivity';
export declare enum ErrorCodes {
    SETUP_FUNCTION = 0,
    RENDER_FUNCTION = 1,
    NATIVE_EVENT_HANDLER = 5,
    COMPONENT_EVENT_HANDLER = 6,
    VNODE_HOOK = 7,
    DIRECTIVE_HOOK = 8,
    TRANSITION_HOOK = 9,
    APP_ERROR_HANDLER = 10,
    APP_WARN_HANDLER = 11,
    FUNCTION_REF = 12,
    ASYNC_COMPONENT_LOADER = 13,
    SCHEDULER = 14,
    COMPONENT_UPDATE = 15,
    APP_UNMOUNT_CLEANUP = 16
}
export declare const ErrorTypeStrings: Record<ErrorTypes, string>;
export type ErrorTypes = LifecycleHooks | ErrorCodes | WatchErrorCodes;
export declare function callWithErrorHandling(fn: Function, instance: ComponentInternalInstance | null | undefined, type: ErrorTypes, args?: unknown[]): any;
export declare function callWithAsyncErrorHandling(fn: Function | Function[], instance: ComponentInternalInstance | null, type: ErrorTypes, args?: unknown[]): any;
export declare function handleError(err: unknown, instance: ComponentInternalInstance | null | undefined, type: ErrorTypes, throwInDev?: boolean): void;
