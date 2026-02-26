import { type NodeTransform, type TransformContext } from '../transform';
import { type DirectiveNode, type ElementNode, type IfBranchNode, type IfNode } from '../ast';
export declare const transformIf: NodeTransform;
export declare function processIf(node: ElementNode, dir: DirectiveNode, context: TransformContext, processCodegen?: (node: IfNode, branch: IfBranchNode, isRoot: boolean) => (() => void) | undefined): (() => void) | undefined;
