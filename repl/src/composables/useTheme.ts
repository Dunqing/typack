import { computed, ref, watchEffect } from "vue";

type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "typack-theme";

// Shared singleton state so all components share the same reactive refs
const preference = ref<ThemePreference>(
  (localStorage.getItem(STORAGE_KEY) as ThemePreference) ?? "system",
);
const systemDark = ref(window.matchMedia("(prefers-color-scheme: dark)").matches);

// Listen for system preference changes (once, at module level)
window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (e) => {
  systemDark.value = e.matches;
});

export function useTheme() {
  const isDark = computed(() =>
    preference.value === "system" ? systemDark.value : preference.value === "dark",
  );

  const monacoTheme = computed(() => (isDark.value ? "vs-dark" : "vs"));

  // Sync .dark class on <html> and persist preference
  watchEffect(() => {
    document.documentElement.classList.toggle("dark", isDark.value);
    localStorage.setItem(STORAGE_KEY, preference.value);
  });

  function toggle() {
    if (preference.value === "system") {
      preference.value = isDark.value ? "light" : "dark";
    } else {
      preference.value = preference.value === "dark" ? "light" : "dark";
    }
  }

  function setPreference(p: ThemePreference) {
    preference.value = p;
  }

  return { isDark, monacoTheme, preference, toggle, setPreference };
}
