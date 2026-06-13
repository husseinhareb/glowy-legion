export type NavigationSection =
  | "dashboard"
  | "effects"
  | "profiles"
  | "diagnostics"
  | "settings";

export interface NavigationItem {
  id: NavigationSection;
  label: string;
}

export const NAVIGATION_ITEMS: NavigationItem[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "effects", label: "Effects" },
  { id: "profiles", label: "Profiles" },
  { id: "diagnostics", label: "Diagnostics" },
  { id: "settings", label: "Settings" },
];
