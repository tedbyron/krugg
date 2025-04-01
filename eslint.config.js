import { fileURLToPath } from 'node:url'

import { includeIgnoreFile } from '@eslint/compat'
import js from '@eslint/js'
import prettier from 'eslint-config-prettier'
import svelte from 'eslint-plugin-svelte'
import globals from 'globals'
import ts from 'typescript-eslint'

export default ts.config(
  includeIgnoreFile(fileURLToPath(new URL('./.gitignore', import.meta.url))),
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
      // Svelte snippets aren't compatible with this.
      '@typescript-eslint/no-confusing-void-expression': 0,
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
  },
)
