import { type ComponentNode, type TransformContext } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare function ssrTransformTransitionGroup(node: ComponentNode, context: TransformContext): () => void;
export declare function ssrProcessTransitionGroup(node: ComponentNode, context: SSRTransformContext): void;
