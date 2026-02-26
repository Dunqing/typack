import type { Node, Positioned } from './esmWalker';
import MagicString from 'magic-string';
export interface HoistMocksOptions {
    /**
     * List of modules that should always be imported before compiler hints.
     * @default 'vitest'
     */
    hoistedModule?: string;
    /**
     * @default ["vi", "vitest"]
     */
    utilsObjectNames?: string[];
    /**
     * @default ["mock", "unmock"]
     */
    hoistableMockMethodNames?: string[];
    /**
     * @default ["mock", "unmock", "doMock", "doUnmock"]
     */
    dynamicImportMockMethodNames?: string[];
    /**
     * @default ["hoisted"]
     */
    hoistedMethodNames?: string[];
    globalThisAccessor?: string;
    regexpHoistable?: RegExp;
    codeFrameGenerator?: CodeFrameGenerator;
    magicString?: () => MagicString;
}
export declare function hoistMocks(code: string, id: string, parse: (code: string) => any, options?: HoistMocksOptions): MagicString | undefined;
interface CodeFrameGenerator {
    (node: Positioned<Node>, id: string, code: string): string;
}
export {};
