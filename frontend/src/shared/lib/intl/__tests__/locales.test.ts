import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import { parse } from '@fluent/syntax';

// Test runs from project root via Vitest
const repoRoot = path.resolve(process.cwd());
const localesDir = path.resolve(repoRoot, 'locales');
const enPath = path.join(localesDir, 'en.ftl');

const REQUIRED_METADATA = ['-lang-code', '-lang-name', '-lang-dialect'];
const tokenRegex = /["']([-a-zA-Z0-9_.]+)["']/g;
const ftlReferenceRegex = /\{\s*([-a-zA-Z0-9]+(?:\.[-a-zA-Z0-9]+)?)\s*\}/g;
const SOURCE_FILE_EXTENSIONS = new Set([
    '.rs',
    '.ts',
    '.tsx',
    '.js',
    '.jsx',
    '.html',
]);
const SKIP_DIRECTORIES = new Set([
    '.git',
    'target',
    'node_modules',
    'dist',
    '.next',
    '.idea',
    '.vscode',
]);

function isRegionalVariant(filename: string) {
    return filename.replace(/\.ftl$/i, '').includes('_');
}

function listLocaleFiles() {
    if (!fs.existsSync(localesDir)) {
        throw new Error(`Locales directory not found: ${localesDir}`);
    }

    return fs
        .readdirSync(localesDir, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith('.ftl'))
        .map((entry) => entry.name)
        .sort();
}

function extractFtlKeys(content: string, filename: string) {
    const keys = new Set<string>();

    const resource = parse(content, {});
    for (const entry of resource.body) {
        if (entry.type === 'Junk') {
            throw new Error(`Invalid FTL syntax in ${filename}`);
        }

        if (entry.type === 'Message') {
            const key = entry.id?.name;
            if (!key) continue;

            if (entry.value) {
                keys.add(key);
            }

            for (const attr of entry.attributes ?? []) {
                if (attr.id?.name) {
                    keys.add(`${key}.${attr.id.name}`);
                }
            }
            continue;
        }

        if (entry.type === 'Term') {
            const key = entry.id?.name ? `-${entry.id.name}` : null;
            if (!key) continue;

            keys.add(key);
            for (const attr of entry.attributes ?? []) {
                if (attr.id?.name) {
                    keys.add(`${key}.${attr.id.name}`);
                }
            }
        }
    }

    return keys;
}

function diff(reference: Set<string>, candidate: Set<string>) {
    const values: string[] = [];
    for (const key of reference) {
        if (!candidate.has(key)) {
            values.push(key);
        }
    }
    return values.sort();
}

function collectSourceFiles(dirPath: string, outFiles: string[]) {
    const entries = fs.readdirSync(dirPath, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(dirPath, entry.name);

        if (entry.isDirectory()) {
            if (SKIP_DIRECTORIES.has(entry.name)) {
                continue;
            }
            collectSourceFiles(fullPath, outFiles);
            continue;
        }

        if (!entry.isFile()) {
            continue;
        }

        const ext = path.extname(entry.name);
        if (SOURCE_FILE_EXTENSIONS.has(ext)) {
            outFiles.push(fullPath);
        }
    }
}

function collectFoundTokens(files: string[]) {
    const found = new Set<string>();
    for (const filePath of files) {
        const content = fs.readFileSync(filePath, 'utf8');
        for (const match of content.matchAll(tokenRegex)) {
            if (match[1]) {
                found.add(match[1]);
            }
        }
    }
    return found;
}

function collectFtlReferencedKeys() {
    const referenced = new Set<string>();
    const files = listLocaleFiles();

    for (const filename of files) {
        const fullPath = path.join(localesDir, filename);
        const content = fs.readFileSync(fullPath, 'utf8');
        for (const match of content.matchAll(ftlReferenceRegex)) {
            if (match[1]) {
                referenced.add(match[1]);
            }
        }
    }

    return referenced;
}

describe('Localization validation', () => {
    it('has the fallback en.ftl file', () => {
        expect(fs.existsSync(enPath)).toBe(true);
    });

    const enKeys = fs.existsSync(enPath)
        ? extractFtlKeys(fs.readFileSync(enPath, 'utf8'), 'en.ftl')
        : new Set<string>();

    const files = fs.existsSync(localesDir) ? listLocaleFiles() : [];

    describe.each(files)('File: %s', (filename) => {
        const fullPath = path.join(localesDir, filename);
        const content = fs.readFileSync(fullPath, 'utf8');
        const keys = extractFtlKeys(content, filename);

        it('has required metadata if not a regional variant', () => {
            if (isRegionalVariant(filename)) {
                return;
            }

            const missingMetadata = REQUIRED_METADATA.filter(
                (key) => !keys.has(key),
            );
            if (missingMetadata.length > 0) {
                throw new Error(
                    `Base locale ${filename} is missing metadata keys:\n` +
                        missingMetadata.map((k) => `  - ${k}`).join('\n'),
                );
            }
        });

        it('has no missing keys compared to en.ftl', () => {
            if (filename === 'en.ftl') return;
            const missing = diff(enKeys, keys);
            if (isRegionalVariant(filename)) return; // Regional variants can have missing keys

            if (missing.length > 0) {
                throw new Error(
                    `Base locale ${filename} is missing keys that exist in en.ftl:\n` +
                        missing.map((k) => `  - ${k}`).join('\n') +
                        '\n\nPlease add these missing translations to ' +
                        filename,
                );
            }
        });

        it('has no extra keys compared to en.ftl', () => {
            if (filename === 'en.ftl') return;
            const extra = diff(keys, enKeys);

            if (extra.length > 0) {
                const type = isRegionalVariant(filename) ? 'Regional' : 'Base';
                throw new Error(
                    `${type} locale ${filename} has keys not present in the fallback en.ftl:\n` +
                        extra.map((k) => `  - ${k}`).join('\n') +
                        '\n\nPlease remove these from ' +
                        filename +
                        ' or add them to en.ftl first.',
                );
            }
        });
    });

    it('has no unused translation keys in en.ftl', () => {
        const sourceFiles: string[] = [];
        collectSourceFiles(repoRoot, sourceFiles);

        const foundTokens = collectFoundTokens(sourceFiles);
        const ftlReferencedKeys = collectFtlReferencedKeys();
        const unused: string[] = [];

        for (const key of enKeys) {
            // Locale metadata terms are not runtime UI keys.
            if (key.startsWith('-lang-')) continue;

            const usedDirectly = foundTokens.has(key);
            const usedPluralVariant =
                foundTokens.has(`${key}_one`) ||
                foundTokens.has(`${key}_other`);
            const usedShortDynamic =
                key.endsWith('.short') &&
                foundTokens.has(
                    key.slice(0, Math.max(0, key.length - '.short'.length)),
                );
            const usedInFtl = ftlReferencedKeys.has(key);

            if (
                !usedDirectly &&
                !usedPluralVariant &&
                !usedShortDynamic &&
                !usedInFtl
            ) {
                unused.push(key);
            }
        }

        if (unused.length > 0) {
            throw new Error(
                'Found unused translation keys in en.ftl:\n' +
                    unused
                        .sort()
                        .map((key) => `  - ${key}`)
                        .join('\n') +
                    '\n\nPlease remove them from locales/en.ftl if they are no longer needed.',
            );
        }
    });
});
