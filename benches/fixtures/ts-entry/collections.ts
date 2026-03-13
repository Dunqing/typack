import type { Disposable, Result } from "./types";

export interface Collection<T> extends Disposable {
  readonly size: number;
  add(item: T): void;
  remove(item: T): boolean;
  has(item: T): boolean;
  clear(): void;
  toArray(): T[];
  forEach(callback: (item: T) => void): void;
}

export interface OrderedCollection<T> extends Collection<T> {
  first(): T | undefined;
  last(): T | undefined;
  at(index: number): T | undefined;
  indexOf(item: T): number;
  sort(comparator: (a: T, b: T) => number): void;
}

export interface ReadonlyCollection<T> {
  readonly size: number;
  has(item: T): boolean;
  toArray(): readonly T[];
  forEach(callback: (item: T) => void): void;
}

export interface MapLike<K, V> extends Disposable {
  readonly size: number;
  get(key: K): V | undefined;
  set(key: K, value: V): void;
  delete(key: K): boolean;
  has(key: K): boolean;
  clear(): void;
  keys(): IterableIterator<K>;
  values(): IterableIterator<V>;
  entries(): IterableIterator<[K, V]>;
}

export type Predicate<T> = (item: T) => boolean;
export type Mapper<T, U> = (item: T) => U;
export type Reducer<T, U> = (accumulator: U, item: T) => U;

export interface QueryableCollection<T> extends Collection<T> {
  filter(predicate: Predicate<T>): T[];
  find(predicate: Predicate<T>): T | undefined;
  map<U>(mapper: Mapper<T, U>): U[];
  reduce<U>(reducer: Reducer<T, U>, initial: U): U;
  some(predicate: Predicate<T>): boolean;
  every(predicate: Predicate<T>): boolean;
}

export interface TreeNode<T> {
  value: T;
  children: TreeNode<T>[];
  parent: TreeNode<T> | null;
  readonly depth: number;
  readonly isLeaf: boolean;
}

export interface Graph<N, E> {
  addNode(node: N): void;
  removeNode(node: N): boolean;
  addEdge(from: N, to: N, edge: E): void;
  removeEdge(from: N, to: N): boolean;
  neighbors(node: N): N[];
  hasPath(from: N, to: N): boolean;
  shortestPath(from: N, to: N): Result<N[], string>;
  readonly nodeCount: number;
  readonly edgeCount: number;
}

export interface PriorityQueue<T> {
  enqueue(item: T, priority: number): void;
  dequeue(): T | undefined;
  peek(): T | undefined;
  readonly size: number;
  readonly isEmpty: boolean;
}

export function createStack<T>(): OrderedCollection<T> {
  const items: T[] = [];
  return {
    get size(): number {
      return items.length;
    },
    add(item: T): void {
      items.push(item);
    },
    remove(item: T): boolean {
      const idx = items.indexOf(item);
      if (idx >= 0) {
        items.splice(idx, 1);
        return true;
      }
      return false;
    },
    has(item: T): boolean {
      return items.includes(item);
    },
    clear(): void {
      items.length = 0;
    },
    toArray(): T[] {
      return [...items];
    },
    forEach(cb: (item: T) => void): void {
      items.forEach(cb);
    },
    first(): T | undefined {
      return items[0];
    },
    last(): T | undefined {
      return items[items.length - 1];
    },
    at(index: number): T | undefined {
      return items[index];
    },
    indexOf(item: T): number {
      return items.indexOf(item);
    },
    sort(comparator: (a: T, b: T) => number): void {
      items.sort(comparator);
    },
    dispose(): void {
      items.length = 0;
    },
  };
}
