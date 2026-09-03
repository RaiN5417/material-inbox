import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { DEFAULT_LOCALE, dictionaries, type Locale, type TranslationKey } from "./locales";

const SETTINGS_KEY = "locale";
const EVENT_LOCALE_CHANGED = "locale-changed";

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: TranslationKey, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key: string) =>
    key in params ? String(params[key]) : match,
  );
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(DEFAULT_LOCALE);

  useEffect(() => {
    invoke<string | null>("get_setting", { key: SETTINGS_KEY })
      .then((value) => {
        if (value === '"zh"' || value === '"en"') {
          setLocaleState(JSON.parse(value) as Locale);
        }
      })
      .catch(() => {});

    const unlisten = listen<Locale>(EVENT_LOCALE_CHANGED, (event) => {
      setLocaleState(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    invoke("set_setting", { key: SETTINGS_KEY, value: JSON.stringify(next) }).catch(() => {});
    // Other windows (e.g. the Floating Card, if open) pick this up live.
    void emit(EVENT_LOCALE_CHANGED, next);
  }, []);

  const t = useCallback(
    (key: TranslationKey, params?: Record<string, string | number>) =>
      interpolate(dictionaries[locale][key] ?? dictionaries[DEFAULT_LOCALE][key] ?? key, params),
    [locale],
  );

  const value = useMemo(() => ({ locale, setLocale, t }), [locale, setLocale, t]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}
