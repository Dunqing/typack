import type { Node as _Node, CallExpression, Function as FunctionNode, Identifier, ImportExpression, Literal, MetaProperty, Property } from 'estree';
import type { Rollup } from 'vite';
export type * from 'estree';
export type Positioned<T> = T & {
    start: number;
    end: number;
};
export type Node = Positioned<_Node>;
interface IdentifierInfo {
    /**
     * If the identifier is used in a property shorthand
     * { foo } -> { foo: __import_x__.foo }
     */
    hasBindingShortcut: boolean;
    /**
     * The identifier is used in a class declaration
     */
    classDeclaration: boolean;
    /**
     * The identifier is a name for a class expression
     */
    classExpression: boolean;
}
interface Visitors {
    onIdentifier?: (node: Positioned<Identifier>, info: IdentifierInfo, parentStack: Node[]) => void;
    onImportMeta?: (node: Positioned<MetaProperty>) => void;
    onDynamicImport?: (node: Positioned<ImportExpression>) => void;
    onCallExpression?: (node: Positioned<CallExpression>) => void;
}
export declare function setIsNodeInPattern(node: Property): WeakSet<_Node>;
export declare function isNodeInPattern(node: _Node): node is Property;
/**
 * Same logic from \@vue/compiler-core & \@vue/compiler-sfc
 * Except this is using acorn AST
 */
export declare function esmWalker(root: ReturnType<Rollup.PluginContext['parse']>, { onIdentifier, onImportMeta, onDynamicImport, onCallExpression }: Visitors): void;
export declare function isStaticProperty(node: _Node): node is Property;
export declare function isStaticPropertyKey(node: _Node, parent: _Node): boolean;
export declare function isFunctionNode(node: _Node): node is FunctionNode;
export declare function isInDestructuringAssignment(parent: _Node, parentStack: _Node[]): boolean;
export declare function getArbitraryModuleIdentifier(node: Identifier | Literal): string;
