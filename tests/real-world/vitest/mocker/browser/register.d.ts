import type { ModuleMockerCompilerHints } from './hints';
import type { ModuleMockerInterceptor } from './index';
import { ModuleMocker } from './index';
export declare function registerModuleMocker(interceptor: (accessor: string) => ModuleMockerInterceptor): ModuleMockerCompilerHints;
export declare function registerNativeFactoryResolver(mocker: ModuleMocker): void;
