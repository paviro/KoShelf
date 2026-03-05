import { FluentBundle, FluentResource } from '@fluent/bundle';
import type { FluentVariable } from '@fluent/bundle';

import { api } from './api';

let bundle: FluentBundle | null = null;
let loadPromise: Promise<void> | null = null;

interface LocalesPayload {
    language: string;
    resources: string[];
}

async function load(): Promise<void> {
    if (bundle) return;

    try {
        const data = await api.locales.get<LocalesPayload>();
        bundle = new FluentBundle(data.language);

        for (const resourceContent of data.resources) {
            const resource = new FluentResource(resourceContent);
            bundle.addResource(resource);
        }
    } catch {
        bundle = new FluentBundle('en-US');
    }
}

export const translation = {
    async init(): Promise<void> {
        if (!loadPromise) {
            loadPromise = load();
        }
        await loadPromise;
    },

    get(key: string, args?: number | Record<string, FluentVariable>): string {
        if (!bundle) return key;

        let fluentArgs: Record<string, FluentVariable> | undefined;
        if (typeof args === 'number') {
            fluentArgs = { count: args };
        } else {
            fluentArgs = args;
        }

        let messageId = key;
        let attributeId: string | undefined;

        const dotIndex = key.indexOf('.');
        if (dotIndex !== -1) {
            messageId = key.substring(0, dotIndex);
            attributeId = key.substring(dotIndex + 1);
        }

        const message = bundle.getMessage(messageId);
        if (!message) return key;

        const pattern = attributeId
            ? message.attributes?.[attributeId]
            : message.value;
        if (!pattern) return key;

        return bundle.formatPattern(pattern, fluentArgs);
    },

    getLanguage(): string {
        return bundle?.locales[0] || 'en-US';
    },
};
