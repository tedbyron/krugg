import { fileURLToPath } from 'node:url'

import { includeIgnoreFile } from '@eslint/compat'
import js from '@eslint/js'
import prettier from 'eslint-config-prettier'
import svelte from 'eslint-plugin-svelte'
import globals from 'globals'
import ts from 'typescript-eslint'

const gitignorePath = fileURLToPath(new URL('./.gitignore', import.meta.url))

export default ts.config(
  includeIgnoreFile(gitignorePath),
  // {
  //   ignores: [
  //     '**/.DS_Store',
  //     '**/node_modules',
  //     '**/build',
  //     '.svelte-kit',
  //     '**/package',
  //     '**/.env',
  //     '**/.env.*',
  //     '!**/.env.example',
  //     '**/package-lock.json',
  //   ],
  // },
  js.configs.recommended,
  ts.configs.eslintRecommended,
  ts.configs.strictTypeChecked,
  ts.configs.stylisticTypeChecked,
  svelte.configs['flat/recommended'],
  prettier,
  svelte.configs['flat/prettier'],
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        ecmaVersion: 'latest',
        ecmaFeatures: { impliedStrict: true },
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        NodeJS: true,
      },
    },
    rules: {
      '@typescript-eslint/no-non-null-assertion': 0,
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        extraFileExtensions: ['.svelte'],
      },
    },
    rules: {
      'svelte/no-at-html-tags': 0,
    },
  },
)
