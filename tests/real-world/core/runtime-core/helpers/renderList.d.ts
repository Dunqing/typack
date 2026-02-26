import type { VNodeChild } from '../vnode';
/**
 * v-for string
 * @private
 */
export declare function renderList(source: string, renderItem: (value: string, index: number) => VNodeChild): VNodeChild[];
/**
 * v-for number
 */
export declare function renderList(source: number, renderItem: (value: number, index: number) => VNodeChild): VNodeChild[];
/**
 * v-for array
 */
export declare function renderList<T>(source: T[], renderItem: (value: T, index: number) => VNodeChild): VNodeChild[];
/**
 * v-for iterable
 */
export declare function renderList<T>(source: Iterable<T>, renderItem: (value: T, index: number) => VNodeChild): VNodeChild[];
/**
 * v-for object
 */
export declare function renderList<T>(source: T, renderItem: <K extends keyof T>(value: T[K], key: string, index: number) => VNodeChild): VNodeChild[];
