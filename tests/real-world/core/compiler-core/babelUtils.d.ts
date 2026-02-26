import type { BlockStatement, Function, Identifier, Node, ObjectProperty, Program, SwitchCase } from '@babel/types';
/**
 * Return value indicates whether the AST walked can be a constant
 */
export declare function walkIdentifiers(root: Node, onIdentifier: (node: Identifier, parent: Node | null, parentStack: Node[], isReference: boolean, isLocal: boolean) => void, includeAll?: boolean, parentStack?: Node[], knownIds?: Record<string, number>): void;
export declare function isReferencedIdentifier(id: Identifier, parent: Node | null, parentStack: Node[]): boolean;
export declare function isInDestructureAssignment(parent: Node, parentStack: Node[]): boolean;
export declare function isInNewExpression(parentStack: Node[]): boolean;
export declare function walkFunctionParams(node: Function, onIdent: (id: Identifier) => void): void;
export declare function walkBlockDeclarations(block: BlockStatement | SwitchCase | Program, onIdent: (node: Identifier) => void): void;
export declare function extractIdentifiers(param: Node, nodes?: Identifier[]): Identifier[];
export declare const isFunctionType: (node: Node) => node is Function;
export declare const isStaticProperty: (node: Node) => node is ObjectProperty;
export declare const isStaticPropertyKey: (node: Node, parent: Node) => boolean;
export declare const TS_NODE_TYPES: string[];
export declare function unwrapTSNode(node: Node): Node;
