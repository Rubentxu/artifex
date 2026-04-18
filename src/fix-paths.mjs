/**
 * Post-build script: rewrite absolute paths to relative in SvelteKit's fallback HTML.
 *
 * SvelteKit's adapter-static always generates absolute paths (/_app/...) in the
 * fallback page, regardless of `paths.relative` or Vite `base` settings.
 * Tauri's custom protocol (tauri://localhost/) cannot resolve these absolute paths,
 * so we convert them to relative (./_app/...) after the build.
 *
 * See: https://svelte.dev/docs/kit/adapter-static#Options-fallback
 */
import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { join } from 'path';

const BUILD_DIR = 'build';

function fixHtmlFile(filePath) {
  let html = readFileSync(filePath, 'utf-8');
  const original = html;

  // Replace absolute /_app/ paths with relative ./_app/
  html = html.replace(/(href|src)="\/_app\//g, '$1="./_app/');
  html = html.replace(/import\("\/_app\//g, 'import("./_app/');

  // Replace absolute /favicon.png with relative ./favicon.png
  html = html.replace(/href="\/favicon\.png"/g, 'href="./favicon.png"');

  // Replace any remaining absolute /assets/ references
  html = html.replace(/(href|src)="\/assets\//g, '$1="./assets/');

  if (html !== original) {
    writeFileSync(filePath, html, 'utf-8');
    console.log(`  ✓ Fixed paths in ${filePath}`);
    return true;
  }
  return false;
}

console.log('Fixing absolute paths in built HTML files...');

let fixed = 0;
for (const file of readdirSync(BUILD_DIR)) {
  if (file.endsWith('.html')) {
    if (fixHtmlFile(join(BUILD_DIR, file))) {
      fixed++;
    }
  }
}

if (fixed === 0) {
  console.log('  (no changes needed)');
} else {
  console.log(`Fixed ${fixed} file(s).`);
}
