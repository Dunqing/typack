import { type ComponentNode, type TransformContext } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare function ssrTransformTransition(node: ComponentNode, context: TransformContext): () => void;
export declare function ssrProcessTransition(node: ComponentNode, context: SSRTransformContext): void;
