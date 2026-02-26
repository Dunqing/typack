import type { AppConfig } from '../apiCreateApp';
export type LegacyConfig = {
    /**
     * @deprecated `config.silent` option has been removed
     */
    silent?: boolean;
    /**
     * @deprecated use __VUE_PROD_DEVTOOLS__ compile-time feature flag instead
     * https://github.com/vuejs/core/tree/main/packages/vue#bundler-build-feature-flags
     */
    devtools?: boolean;
    /**
     * @deprecated use `config.isCustomElement` instead
     * https://v3-migration.vuejs.org/breaking-changes/global-api.html#config-ignoredelements-is-now-config-iscustomelement
     */
    ignoredElements?: (string | RegExp)[];
    /**
     * @deprecated
     * https://v3-migration.vuejs.org/breaking-changes/keycode-modifiers.html
     */
    keyCodes?: Record<string, number | number[]>;
    /**
     * @deprecated
     * https://v3-migration.vuejs.org/breaking-changes/global-api.html#config-productiontip-removed
     */
    productionTip?: boolean;
};
export declare function installLegacyConfigWarnings(config: AppConfig): void;
export declare function installLegacyOptionMergeStrats(config: AppConfig): void;
