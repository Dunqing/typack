import type { MockedModule } from '../registry';
import type { ModuleMockerInterceptor } from './interceptor';
export declare class ModuleMockerServerInterceptor implements ModuleMockerInterceptor {
    register(module: MockedModule): Promise<void>;
    delete(id: string): Promise<void>;
    invalidate(): Promise<void>;
}
