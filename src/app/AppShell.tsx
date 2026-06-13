import { useEffect, useState } from "react";

import { applyProfile } from "../api/profileApi";
import { DashboardPage } from "../features/dashboard/DashboardPage";
import { DiagnosticsPage } from "../features/diagnostics/DiagnosticsPage";
import { LightingPage } from "../features/lighting/LightingPage";
import { ProfilesPage } from "../features/profiles/ProfilesPage";
import { SettingsPage } from "../features/settings/SettingsPage";
import { Notice } from "../shared/components/Notice";
import { StatusBadge } from "../shared/components/StatusBadge";
import { useKeyboardState } from "../shared/hooks/useKeyboardState";
import { useToast } from "../shared/hooks/useToast";
import { useAppStore } from "../state/appStore";
import { useKeyboardStore } from "../state/keyboardStore";
import { useProfileStore } from "../state/profileStore";
import { APP_DISPLAY_NAME } from "./constants";
import {
  NAVIGATION_ITEMS,
  type NavigationSection,
} from "./navigation";

export function AppShell() {
  const [activeSection, setActiveSection] =
    useState<NavigationSection>("dashboard");
  const [actionLoading, setActionLoading] = useState(false);
  const app = useAppStore();
  const keyboardStore = useKeyboardStore();
  const keyboard = useKeyboardState(keyboardStore);
  const profiles = useProfileStore();
  const { toast, showToast, clearToast } = useToast();

  useEffect(() => {
    app.loadAppInfo().catch((error) => {
      showToast("error", errorToMessage(error));
    });
    keyboard.refresh().catch((error) => {
      showToast("error", errorToMessage(error));
    });
    profiles.loadProfiles().catch((error) => {
      showToast("error", errorToMessage(error));
    });
  }, [app.loadAppInfo, keyboard.refresh, profiles.loadProfiles, showToast]);

  const busy = app.loading || keyboard.loading || profiles.loading || actionLoading;

  const runAction = async (task: () => Promise<void>, successMessage: string) => {
    setActionLoading(true);
    clearToast();

    try {
      await task();
      showToast("success", successMessage);
    } catch (error) {
      showToast("error", errorToMessage(error));
    } finally {
      setActionLoading(false);
    }
  };

  const storeErrors = [app.error, keyboard.error, profiles.error].filter(
    (message): message is string => Boolean(message),
  );

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-block">
          <div className="brand-mark">LG</div>
          <div>
            <strong>{app.appInfo?.name ?? APP_DISPLAY_NAME}</strong>
            <span>Linux RGB control</span>
          </div>
        </div>
        <nav className="sidebar-nav" aria-label="Primary">
          {NAVIGATION_ITEMS.map((item) => (
            <button
              className={item.id === activeSection ? "is-active" : ""}
              key={item.id}
              type="button"
              onClick={() => setActiveSection(item.id)}
            >
              {item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <StatusBadge
            label={
              app.appInfo?.backend_mode === "lenovo-hid"
                ? "Lenovo HID backend"
                : app.appInfo?.backend_mode === "lenovo-hid-dry-run"
                  ? "HID dry-run backend"
                : "Mock backend active"
            }
            tone={app.appInfo?.real_hardware_writes_enabled ? "danger" : "ok"}
          />
          <small>{keyboard.device?.product_name ?? "Detecting mock device"}</small>
        </div>
      </aside>

      <main className="main-panel">
        {toast && (
          <button
            className={`toast toast--${toast.kind}`}
            type="button"
            onClick={clearToast}
          >
            {toast.message}
          </button>
        )}

        {storeErrors.map((message) => (
          <Notice key={message} tone="error">
            {message}
          </Notice>
        ))}

        {activeSection === "dashboard" && (
          <DashboardPage
            appInfo={app.appInfo}
            device={keyboard.device}
            keyboardState={keyboard.keyboardState}
            loading={busy}
            onApplyCurrent={() => {
              const currentState = keyboard.keyboardState;

              if (!currentState) {
                return;
              }

              runAction(
                async () => {
                  await keyboard.applyState(currentState);
                },
                "Lighting mode applied.",
              );
            }}
            onRefresh={() =>
              runAction(
                async () => {
                  await keyboard.refresh();
                },
                "Device state refreshed.",
              )
            }
            onTurnOff={() =>
              runAction(
                async () => {
                  await keyboard.turnOff();
                },
                "Backlight turned off.",
              )
            }
          />
        )}

        {activeSection === "effects" && (
          <LightingPage
            device={keyboard.device}
            keyboardState={keyboard.keyboardState}
            loading={busy}
            onApply={(state) =>
              runAction(
                async () => {
                  await keyboard.applyState(state);
                },
                "Lighting settings applied.",
              )
            }
            onTurnOff={() =>
              runAction(
                async () => {
                  await keyboard.turnOff();
                },
                "Backlight turned off.",
              )
            }
          />
        )}

        {activeSection === "profiles" && (
          <ProfilesPage
            device={keyboard.device}
            loading={busy}
            profiles={profiles.profiles}
            onApplyProfile={(profileId) =>
              runAction(
                async () => {
                  const updatedState = await applyProfile(profileId);
                  keyboard.replaceKeyboardState(updatedState);
                },
                "Profile applied.",
              )
            }
          />
        )}

        {activeSection === "diagnostics" && <DiagnosticsPage />}

        {activeSection === "settings" && (
          <SettingsPage appInfo={app.appInfo} device={keyboard.device} />
        )}
      </main>
    </div>
  );
}

function errorToMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error ?? "Unknown error");
}
