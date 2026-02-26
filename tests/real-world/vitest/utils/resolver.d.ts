export declare function findNearestPackageData(basedir: string): {
    type?: 'module' | 'commonjs';
};
export declare function getCachedData<T>(cache: Map<string, T>, basedir: string, originalBasedir: string): NonNullable<T> | undefined;
export declare function setCacheData<T>(cache: Map<string, T>, data: T, basedir: string, originalBasedir: string): void;
