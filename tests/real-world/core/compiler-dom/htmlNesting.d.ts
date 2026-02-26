/**
 * Copied from https://github.com/MananTank/validate-html-nesting
 * with ISC license
 *
 * To avoid runtime dependency on validate-html-nesting
 * This file should not change very often in the original repo
 * but we may need to keep it up-to-date from time to time.
 */
/**
 * returns true if given parent-child nesting is valid HTML
 */
export declare function isValidHTMLNesting(parent: string, child: string): boolean;
