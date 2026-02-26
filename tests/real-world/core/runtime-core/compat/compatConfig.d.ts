import { type Component, type ComponentInternalInstance } from '../component';
export declare enum DeprecationTypes {
    GLOBAL_MOUNT = "GLOBAL_MOUNT",
    GLOBAL_MOUNT_CONTAINER = "GLOBAL_MOUNT_CONTAINER",
    GLOBAL_EXTEND = "GLOBAL_EXTEND",
    GLOBAL_PROTOTYPE = "GLOBAL_PROTOTYPE",
    GLOBAL_SET = "GLOBAL_SET",
    GLOBAL_DELETE = "GLOBAL_DELETE",
    GLOBAL_OBSERVABLE = "GLOBAL_OBSERVABLE",
    GLOBAL_PRIVATE_UTIL = "GLOBAL_PRIVATE_UTIL",
    CONFIG_SILENT = "CONFIG_SILENT",
    CONFIG_DEVTOOLS = "CONFIG_DEVTOOLS",
    CONFIG_KEY_CODES = "CONFIG_KEY_CODES",
    CONFIG_PRODUCTION_TIP = "CONFIG_PRODUCTION_TIP",
    CONFIG_IGNORED_ELEMENTS = "CONFIG_IGNORED_ELEMENTS",
    CONFIG_WHITESPACE = "CONFIG_WHITESPACE",
    CONFIG_OPTION_MERGE_STRATS = "CONFIG_OPTION_MERGE_STRATS",
    INSTANCE_SET = "INSTANCE_SET",
    INSTANCE_DELETE = "INSTANCE_DELETE",
    INSTANCE_DESTROY = "INSTANCE_DESTROY",
    INSTANCE_EVENT_EMITTER = "INSTANCE_EVENT_EMITTER",
    INSTANCE_EVENT_HOOKS = "INSTANCE_EVENT_HOOKS",
    INSTANCE_CHILDREN = "INSTANCE_CHILDREN",
    INSTANCE_LISTENERS = "INSTANCE_LISTENERS",
    INSTANCE_SCOPED_SLOTS = "INSTANCE_SCOPED_SLOTS",
    INSTANCE_ATTRS_CLASS_STYLE = "INSTANCE_ATTRS_CLASS_STYLE",
    OPTIONS_DATA_FN = "OPTIONS_DATA_FN",
    OPTIONS_DATA_MERGE = "OPTIONS_DATA_MERGE",
    OPTIONS_BEFORE_DESTROY = "OPTIONS_BEFORE_DESTROY",
    OPTIONS_DESTROYED = "OPTIONS_DESTROYED",
    WATCH_ARRAY = "WATCH_ARRAY",
    PROPS_DEFAULT_THIS = "PROPS_DEFAULT_THIS",
    V_ON_KEYCODE_MODIFIER = "V_ON_KEYCODE_MODIFIER",
    CUSTOM_DIR = "CUSTOM_DIR",
    ATTR_FALSE_VALUE = "ATTR_FALSE_VALUE",
    ATTR_ENUMERATED_COERCION = "ATTR_ENUMERATED_COERCION",
    TRANSITION_CLASSES = "TRANSITION_CLASSES",
    TRANSITION_GROUP_ROOT = "TRANSITION_GROUP_ROOT",
    COMPONENT_ASYNC = "COMPONENT_ASYNC",
    COMPONENT_FUNCTIONAL = "COMPONENT_FUNCTIONAL",
    COMPONENT_V_MODEL = "COMPONENT_V_MODEL",
    RENDER_FUNCTION = "RENDER_FUNCTION",
    FILTERS = "FILTERS",
    PRIVATE_APIS = "PRIVATE_APIS"
}
type DeprecationData = {
    message: string | ((...args: any[]) => string);
    link?: string;
};
export declare const deprecationData: Record<DeprecationTypes, DeprecationData>;
export declare function toggleDeprecationWarning(flag: boolean): void;
export declare function warnDeprecation(key: DeprecationTypes, instance: ComponentInternalInstance | null, ...args: any[]): void;
export type CompatConfig = Partial<Record<DeprecationTypes, boolean | 'suppress-warning'>> & {
    MODE?: 2 | 3 | ((comp: Component | null) => 2 | 3);
};
export declare const globalCompatConfig: CompatConfig;
export declare function configureCompat(config: CompatConfig): void;
export declare function validateCompatConfig(config: CompatConfig, instance?: ComponentInternalInstance): void;
export declare function getCompatConfigForKey(key: DeprecationTypes | 'MODE', instance: ComponentInternalInstance | null): CompatConfig[DeprecationTypes | 'MODE'];
export declare function isCompatEnabled(key: DeprecationTypes, instance: ComponentInternalInstance | null, enableForBuiltIn?: boolean): boolean;
/**
 * Use this for features that are completely removed in non-compat build.
 */
export declare function assertCompatEnabled(key: DeprecationTypes, instance: ComponentInternalInstance | null, ...args: any[]): void;
/**
 * Use this for features where legacy usage is still possible, but will likely
 * lead to runtime error if compat is disabled. (warn in all cases)
 */
export declare function softAssertCompatEnabled(key: DeprecationTypes, instance: ComponentInternalInstance | null, ...args: any[]): boolean;
/**
 * Use this for features with the same syntax but with mutually exclusive
 * behavior in 2 vs 3. Only warn if compat is enabled.
 * e.g. render function
 */
export declare function checkCompatEnabled(key: DeprecationTypes, instance: ComponentInternalInstance | null, ...args: any[]): boolean;
export {};
