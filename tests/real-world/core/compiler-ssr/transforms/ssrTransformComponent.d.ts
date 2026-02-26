import { type CompilerOptions, type ComponentNode, type NodeTransform, type RootNode, type TemplateChildNode } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare const ssrTransformComponent: NodeTransform;
export declare function ssrProcessComponent(node: ComponentNode, context: SSRTransformContext, parent: {
    children: TemplateChildNode[];
}): void;
export declare const rawOptionsMap: WeakMap<RootNode, CompilerOptions>;
