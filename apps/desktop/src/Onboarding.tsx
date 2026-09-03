import { useEffect, useState, type ReactNode } from "react";
import { useI18n } from "./i18n/context";
import type { TranslationKey } from "./i18n/locales";
import { CloseIcon, GalleryViewIcon, GroupsIcon, InboxIcon, SettingsIcon, TagIcon } from "./icons";

interface Step {
  icon: ReactNode;
  titleKey: TranslationKey;
  bodyKey: TranslationKey;
}

const STEPS: Step[] = [
  {
    icon: <InboxIcon width={30} height={30} />,
    titleKey: "onboarding.step1.title",
    bodyKey: "onboarding.step1.body",
  },
  {
    icon: <GalleryViewIcon width={30} height={30} />,
    titleKey: "onboarding.step2.title",
    bodyKey: "onboarding.step2.body",
  },
  {
    icon: <GroupsIcon width={30} height={30} />,
    titleKey: "onboarding.step3.title",
    bodyKey: "onboarding.step3.body",
  },
  {
    icon: <TagIcon width={30} height={30} />,
    titleKey: "onboarding.step4.title",
    bodyKey: "onboarding.step4.body",
  },
  {
    icon: <SettingsIcon width={30} height={30} />,
    titleKey: "onboarding.step5.title",
    bodyKey: "onboarding.step5.body",
  },
];

// First-launch tour — a handful of static steps (no template/holes needed)
// walking through the parts of the app that aren't self-explanatory from
// the UI alone. Reopenable any time from Settings.
export function Onboarding({ onClose }: { onClose: (dontShowAgain: boolean) => void }) {
  const { t } = useI18n();
  const [index, setIndex] = useState(0);
  const [dontShowAgain, setDontShowAgain] = useState(true);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose(dontShowAgain);
      if (e.key === "ArrowRight") setIndex((i) => Math.min(i + 1, STEPS.length - 1));
      if (e.key === "ArrowLeft") setIndex((i) => Math.max(i - 1, 0));
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [dontShowAgain, onClose]);

  const step = STEPS[index];
  const isLast = index === STEPS.length - 1;

  return (
    <div className="onboarding-backdrop" onClick={() => onClose(dontShowAgain)}>
      <div
        className="onboarding-panel"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          className="onboarding-close"
          aria-label={t("common.close")}
          onClick={() => onClose(dontShowAgain)}
        >
          <CloseIcon width={14} height={14} />
        </button>

        <div className="onboarding-icon">{step.icon}</div>
        <h2 className="onboarding-title">{t(step.titleKey)}</h2>
        <p className="onboarding-body">{t(step.bodyKey)}</p>

        <div className="onboarding-dots">
          {STEPS.map((_, i) => (
            <span key={i} className={`onboarding-dot ${i === index ? "active" : ""}`} />
          ))}
        </div>

        <label className="onboarding-checkbox">
          <input
            type="checkbox"
            checked={dontShowAgain}
            onChange={(e) => setDontShowAgain(e.target.checked)}
          />
          {t("onboarding.dontShowAgain")}
        </label>

        <div className="onboarding-actions">
          {index > 0 ? (
            <button className="btn-secondary" onClick={() => setIndex((i) => i - 1)}>
              {t("onboarding.back")}
            </button>
          ) : (
            <button className="btn-secondary" onClick={() => onClose(dontShowAgain)}>
              {t("onboarding.skip")}
            </button>
          )}
          <button
            className="btn-primary"
            onClick={() => (isLast ? onClose(dontShowAgain) : setIndex((i) => i + 1))}
          >
            {isLast ? t("onboarding.finish") : t("onboarding.next")}
          </button>
        </div>
      </div>
    </div>
  );
}
