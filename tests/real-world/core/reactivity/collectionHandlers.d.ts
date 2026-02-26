import { type Target } from './reactive';
type CollectionTypes = IterableCollections | WeakCollections;
type IterableCollections = (Map<any, any> | Set<any>) & Target;
type WeakCollections = (WeakMap<any, any> | WeakSet<any>) & Target;
export declare const mutableCollectionHandlers: ProxyHandler<CollectionTypes>;
export declare const shallowCollectionHandlers: ProxyHandler<CollectionTypes>;
export declare const readonlyCollectionHandlers: ProxyHandler<CollectionTypes>;
export declare const shallowReadonlyCollectionHandlers: ProxyHandler<CollectionTypes>;
export {};
