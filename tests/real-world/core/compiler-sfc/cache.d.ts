import { LRUCache } from 'lru-cache';
export declare function createCache<T extends {}>(max?: number): Map<string, T> | LRUCache<string, T>;
