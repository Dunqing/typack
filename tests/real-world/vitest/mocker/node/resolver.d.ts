import type { ViteDevServer } from 'vite';
import type { ServerIdResolution, ServerMockResolution } from '../types';
export interface ServerResolverOptions {
    /**
     * @default ['/node_modules/']
     */
    moduleDirectories?: string[];
}
export declare class ServerMockResolver {
    private server;
    private options;
    constructor(server: ViteDevServer, options?: ServerResolverOptions);
    resolveMock(rawId: string, importer: string, options: {
        mock: 'spy' | 'factory' | 'auto';
    }): Promise<ServerMockResolution>;
    invalidate(ids: string[]): void;
    resolveId(id: string, importer?: string): Promise<ServerIdResolution | null>;
    private normalizeResolveIdToUrl;
    private resolveMockId;
    private resolveModule;
}
