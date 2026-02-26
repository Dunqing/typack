/**
 * This Tokenizer is adapted from htmlparser2 under the MIT License listed at
 * https://github.com/fb55/htmlparser2/blob/master/LICENSE

Copyright 2010, 2011, Chris Winberry <chris@winberry.net>. All rights reserved.
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to
deal in the Software without restriction, including without limitation the
rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
IN THE SOFTWARE.
 */
import { ErrorCodes } from './errors';
import type { ElementNode, Position } from './ast';
export declare enum ParseMode {
    BASE = 0,
    HTML = 1,
    SFC = 2
}
export declare enum CharCodes {
    Tab = 9,// "\t"
    NewLine = 10,// "\n"
    FormFeed = 12,// "\f"
    CarriageReturn = 13,// "\r"
    Space = 32,// " "
    ExclamationMark = 33,// "!"
    Number = 35,// "#"
    Amp = 38,// "&"
    SingleQuote = 39,// "'"
    DoubleQuote = 34,// '"'
    GraveAccent = 96,// "`"
    Dash = 45,// "-"
    Slash = 47,// "/"
    Zero = 48,// "0"
    Nine = 57,// "9"
    Semi = 59,// ";"
    Lt = 60,// "<"
    Eq = 61,// "="
    Gt = 62,// ">"
    Questionmark = 63,// "?"
    UpperA = 65,// "A"
    LowerA = 97,// "a"
    UpperF = 70,// "F"
    LowerF = 102,// "f"
    UpperZ = 90,// "Z"
    LowerZ = 122,// "z"
    LowerX = 120,// "x"
    LowerV = 118,// "v"
    Dot = 46,// "."
    Colon = 58,// ":"
    At = 64,// "@"
    LeftSquare = 91,// "["
    RightSquare = 93
}
/** All the states the tokenizer can be in. */
export declare enum State {
    Text = 1,
    InterpolationOpen = 2,
    Interpolation = 3,
    InterpolationClose = 4,
    BeforeTagName = 5,// After <
    InTagName = 6,
    InSelfClosingTag = 7,
    BeforeClosingTagName = 8,
    InClosingTagName = 9,
    AfterClosingTagName = 10,
    BeforeAttrName = 11,
    InAttrName = 12,
    InDirName = 13,
    InDirArg = 14,
    InDirDynamicArg = 15,
    InDirModifier = 16,
    AfterAttrName = 17,
    BeforeAttrValue = 18,
    InAttrValueDq = 19,// "
    InAttrValueSq = 20,// '
    InAttrValueNq = 21,
    BeforeDeclaration = 22,// !
    InDeclaration = 23,
    InProcessingInstruction = 24,// ?
    BeforeComment = 25,
    CDATASequence = 26,
    InSpecialComment = 27,
    InCommentLike = 28,
    BeforeSpecialS = 29,// Decide if we deal with `<script` or `<style`
    BeforeSpecialT = 30,// Decide if we deal with `<title` or `<textarea`
    SpecialStartSequence = 31,
    InRCDATA = 32,
    InEntity = 33,
    InSFCRootTagName = 34
}
export declare function isWhitespace(c: number): boolean;
export declare function toCharCodes(str: string): Uint8Array;
export declare enum QuoteType {
    NoValue = 0,
    Unquoted = 1,
    Single = 2,
    Double = 3
}
export interface Callbacks {
    ontext(start: number, endIndex: number): void;
    ontextentity(char: string, start: number, endIndex: number): void;
    oninterpolation(start: number, endIndex: number): void;
    onopentagname(start: number, endIndex: number): void;
    onopentagend(endIndex: number): void;
    onselfclosingtag(endIndex: number): void;
    onclosetag(start: number, endIndex: number): void;
    onattribdata(start: number, endIndex: number): void;
    onattribentity(char: string, start: number, end: number): void;
    onattribend(quote: QuoteType, endIndex: number): void;
    onattribname(start: number, endIndex: number): void;
    onattribnameend(endIndex: number): void;
    ondirname(start: number, endIndex: number): void;
    ondirarg(start: number, endIndex: number): void;
    ondirmodifier(start: number, endIndex: number): void;
    oncomment(start: number, endIndex: number): void;
    oncdata(start: number, endIndex: number): void;
    onprocessinginstruction(start: number, endIndex: number): void;
    onend(): void;
    onerr(code: ErrorCodes, index: number): void;
}
/**
 * Sequences used to match longer strings.
 *
 * We don't have `Script`, `Style`, or `Title` here. Instead, we re-use the *End
 * sequences with an increased offset.
 */
export declare const Sequences: {
    Cdata: Uint8Array;
    CdataEnd: Uint8Array;
    CommentEnd: Uint8Array;
    ScriptEnd: Uint8Array;
    StyleEnd: Uint8Array;
    TitleEnd: Uint8Array;
    TextareaEnd: Uint8Array;
};
export default class Tokenizer {
    private readonly stack;
    private readonly cbs;
    /** The current state the tokenizer is in. */
    state: State;
    /** The read buffer. */
    private buffer;
    /** The beginning of the section that is currently being read. */
    sectionStart: number;
    /** The index within the buffer that we are currently looking at. */
    private index;
    /** The start of the last entity. */
    private entityStart;
    /** Some behavior, eg. when decoding entities, is done while we are in another state. This keeps track of the other state type. */
    private baseState;
    /** For special parsing behavior inside of script and style tags. */
    inRCDATA: boolean;
    /** For disabling RCDATA tags handling */
    inXML: boolean;
    /** For disabling interpolation parsing in v-pre */
    inVPre: boolean;
    /** Record newline positions for fast line / column calculation */
    private newlines;
    private readonly entityDecoder?;
    mode: ParseMode;
    get inSFCRoot(): boolean;
    constructor(stack: ElementNode[], cbs: Callbacks);
    reset(): void;
    /**
     * Generate Position object with line / column information using recorded
     * newline positions. We know the index is always going to be an already
     * processed index, so all the newlines up to this index should have been
     * recorded.
     */
    getPos(index: number): Position;
    private peek;
    private stateText;
    delimiterOpen: Uint8Array;
    delimiterClose: Uint8Array;
    private delimiterIndex;
    private stateInterpolationOpen;
    private stateInterpolation;
    private stateInterpolationClose;
    currentSequence: Uint8Array;
    private sequenceIndex;
    private stateSpecialStartSequence;
    /** Look for an end tag. For <title> and <textarea>, also decode entities. */
    private stateInRCDATA;
    private stateCDATASequence;
    /**
     * When we wait for one specific character, we can speed things up
     * by skipping through the buffer until we find it.
     *
     * @returns Whether the character was found.
     */
    private fastForwardTo;
    /**
     * Comments and CDATA end with `-->` and `]]>`.
     *
     * Their common qualities are:
     * - Their end sequences have a distinct character they start with.
     * - That character is then repeated, so we have to check multiple repeats.
     * - All characters but the start character of the sequence can be skipped.
     */
    private stateInCommentLike;
    private startSpecial;
    enterRCDATA(sequence: Uint8Array, offset: number): void;
    private stateBeforeTagName;
    private stateInTagName;
    private stateInSFCRootTagName;
    private handleTagName;
    private stateBeforeClosingTagName;
    private stateInClosingTagName;
    private stateAfterClosingTagName;
    private stateBeforeAttrName;
    private handleAttrStart;
    private stateInSelfClosingTag;
    private stateInAttrName;
    private stateInDirName;
    private stateInDirArg;
    private stateInDynamicDirArg;
    private stateInDirModifier;
    private handleAttrNameEnd;
    private stateAfterAttrName;
    private stateBeforeAttrValue;
    private handleInAttrValue;
    private stateInAttrValueDoubleQuotes;
    private stateInAttrValueSingleQuotes;
    private stateInAttrValueNoQuotes;
    private stateBeforeDeclaration;
    private stateInDeclaration;
    private stateInProcessingInstruction;
    private stateBeforeComment;
    private stateInSpecialComment;
    private stateBeforeSpecialS;
    private stateBeforeSpecialT;
    private startEntity;
    private stateInEntity;
    /**
     * Iterates through the buffer, calling the function corresponding to the current state.
     *
     * States that are more likely to be hit are higher up, as a performance improvement.
     */
    parse(input: string): void;
    /**
     * Remove data that has already been consumed from the buffer.
     */
    private cleanup;
    private finish;
    /** Handle any trailing data. */
    private handleTrailingData;
    private emitCodePoint;
}
