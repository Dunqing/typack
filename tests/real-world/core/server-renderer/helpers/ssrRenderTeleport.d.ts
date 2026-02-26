import { type ComponentInternalInstance } from 'vue';
import { type PushFn } from '../render';
export declare function ssrRenderTeleport(parentPush: PushFn, contentRenderFn: (push: PushFn) => void, target: string, disabled: boolean, parentComponent: ComponentInternalInstance): void;
