# typack

A native TypeScript declaration (`.d.ts`) bundler built on [Oxc](https://oxc.rs).

## Install

```sh
npm install typack
```

> Requires Node.js >= 20.

## CLI

```sh
npx typack [options] <entry.d.ts...>
```

### Options

| Flag                   | Description                                     |
| ---------------------- | ----------------------------------------------- |
| `--external <SPEC>`    | Module specifiers to keep external (repeatable) |
| `--cwd <DIR>`          | Working directory (default: current directory)  |
| `--sourcemap`          | Generate source map (`.d.ts.map`)               |
| `--cjs-default`        | Emit `export =` for single default export       |
| `-o, --outfile <PATH>` | Write output to file instead of stdout          |

### Example

```sh
npx typack --external react --outfile dist/index.d.ts src/index.d.ts
```

## Programmatic API

```js
const { bundle } = require("typack");

const result = bundle({
  input: ["src/index.d.ts"],
  external: ["react"],
  cwd: process.cwd(),
  sourcemap: true,
  cjsDefault: false,
});

console.log(result.code);
// result.map   - source map string (if sourcemap: true)
// result.warnings - array of { message, severity }
```

### `bundle(options): BundleDtsResult`

#### Options

| Field        | Type       | Required | Description                               |
| ------------ | ---------- | -------- | ----------------------------------------- |
| `input`      | `string[]` | Yes      | Entry `.d.ts` files to bundle             |
| `external`   | `string[]` | No       | Module specifiers to keep external        |
| `cwd`        | `string`   | No       | Working directory                         |
| `sourcemap`  | `boolean`  | No       | Generate source map                       |
| `cjsDefault` | `boolean`  | No       | Emit `export =` for single default export |

#### Result

| Field      | Type                    | Description            |
| ---------- | ----------------------- | ---------------------- |
| `code`     | `string`                | Bundled `.d.ts` output |
| `map`      | `string \| undefined`   | Source map JSON string |
| `warnings` | `BundleDtsDiagnostic[]` | Array of warnings      |

## License

MIT
