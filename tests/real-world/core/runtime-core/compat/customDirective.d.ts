import type { ComponentInternalInstance } from '../component';
import type { DirectiveHook, ObjectDirective } from '../directives';
export interface LegacyDirective {
    bind?: DirectiveHook;
    inserted?: DirectiveHook;
    update?: DirectiveHook;
    componentUpdated?: DirectiveHook;
    unbind?: DirectiveHook;
}
export declare function mapCompatDirectiveHook(name: keyof ObjectDirective, dir: ObjectDirective & LegacyDirective, instance: ComponentInternalInstance | null): DirectiveHook | DirectiveHook[] | undefined;
