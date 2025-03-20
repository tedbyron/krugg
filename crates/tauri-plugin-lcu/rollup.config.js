import path from 'node:path'

import typescript from '@rollup/plugin-typescript'

import pkg from './package.json' with { type: 'json' }

const prod = process.env.NODE_ENV === 'production'

/** @satisfies {import('rollup').RollupOptions} */
export default {
  input: 'lib/index.ts',
  output: [
    {
      file: pkg.exports.import,
      format: 'esm',
      sourcemap: !prod,
    },
    {
      file: pkg.exports.require,
      format: 'cjs',
      sourcemap: !prod,
    },
  ],
  plugins: [
    typescript({
      declaration: true,
      declarationDir: path.parse(pkg.exports.types).dir,
      sourceMap: !prod,
      outputToFilesystem: true,
    }),
  ],
  external: [
    /^@tauri-apps\/api/,
    ...Object.keys(pkg.dependencies),
    // ...Object.keys(pkg.peerDependencies),
  ],
}
