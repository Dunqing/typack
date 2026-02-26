import MagicString from 'magic-string';
export interface AutomockOptions {
    /**
     * @default "__vitest_mocker__"
     */
    globalThisAccessor?: string;
    id?: string;
}
export declare function automockModule(code: string, mockType: 'automock' | 'autospy', parse: (code: string) => any, options?: AutomockOptions): MagicString;
