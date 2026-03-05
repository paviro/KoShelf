import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from '@fluent/syntax';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..', '..');
const localesDir = path.resolve(__dirname, '..', 'locales');
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

function fail(message) {
    throw new Error(message);
}

function isRegionalVariant(filename) {
    return filename.replace(/\.ftl$/i, '').includes('_');
}

function listLocaleFiles() {
    if (!fs.existsSync(localesDir)) {
        fail(`Locales directory not found: ${localesDir}`);
    }

    return fs
        .readdirSync(localesDir, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith('.ftl'))
        .map((entry) => entry.name)
        .sort();
}

function extractFtlKeys(content, filename) {
    const keys = new Set();

    const resource = parse(content);
    for (const entry of resource.body) {
        if (entry.type === 'Junk') {
            fail(`Invalid FTL syntax in ${filename}`);
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

function diff(reference, candidate) {
    const values = [];
    for (const key of reference) {
        if (!candidate.has(key)) {
            values.push(key);
        }
    }
    return values.sort();
}

function validateMetadata(filename, keys) {
    if (isRegionalVariant(filename)) {
        return;
    }

    const missingMetadata = REQUIRED_METADATA.filter((key) => !keys.has(key));
    if (missingMetadata.length > 0) {
        fail(
            `Base locale ${filename} is missing metadata keys: ${missingMetadata.join(', ')}`,
        );
    }
}

function validateKeySet(filename, enKeys, localeKeys) {
    if (filename === 'en.ftl') {
        return;
    }

    const missing = diff(enKeys, localeKeys);
    const extra = diff(localeKeys, enKeys);

    if (isRegionalVariant(filename)) {
        if (extra.length > 0) {
            fail(
                `Regional locale ${filename} has keys not present in en.ftl: ${extra.join(', ')}`,
            );
        }
        return;
    }

    if (missing.length > 0) {
        fail(`Base locale ${filename} is missing keys: ${missing.join(', ')}`);
    }

    if (extra.length > 0) {
        fail(
            `Base locale ${filename} has keys not present in en.ftl: ${extra.join(', ')}`,
        );
    }
}

function collectSourceFiles(dirPath, outFiles) {
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

function collectFoundTokens(files) {
    const found = new Set();
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
    const referenced = new Set();
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

function validateNoUnusedTranslationKeys(enKeys) {
    const sourceFiles = [];
    collectSourceFiles(repoRoot, sourceFiles);

    const foundTokens = collectFoundTokens(sourceFiles);
    const ftlReferencedKeys = collectFtlReferencedKeys();
    const unused = [];

    for (const key of enKeys) {
        // Locale metadata terms are not runtime UI keys.
        if (key.startsWith('-lang-')) {
            continue;
        }

        const usedDirectly = foundTokens.has(key);
        const usedPluralVariant =
            foundTokens.has(`${key}_one`) || foundTokens.has(`${key}_other`);
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
        unused.sort();
        fail(`Found unused translation keys in en.ftl: ${unused.join(', ')}`);
    }
}

function main() {
    if (!fs.existsSync(enPath)) {
        fail(`Required fallback locale not found: ${enPath}`);
    }

    const enKeys = extractFtlKeys(fs.readFileSync(enPath, 'utf8'), 'en.ftl');
    const files = listLocaleFiles();

    for (const filename of files) {
        const fullPath = path.join(localesDir, filename);
        const content = fs.readFileSync(fullPath, 'utf8');
        const keys = extractFtlKeys(content, filename);
        validateMetadata(filename, keys);
        validateKeySet(filename, enKeys, keys);
    }

    validateNoUnusedTranslationKeys(enKeys);

    console.log(`Locale validation passed for ${files.length} locale files.`);
}

main();
