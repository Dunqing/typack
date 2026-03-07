import { computed, ref, watchEffect } from "vue";

type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "typack-theme";
const VALID_PREFS = new Set<ThemePreference>(["system", "light", "dark"]);

function readStoredPreference(): ThemePreference {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw && VALID_PREFS.has(raw as ThemePreference)) {
      return raw as ThemePreference;
    }
  } catch {
    // localStorage may be unavailable (private browsing, blocked storage)
  }
  return "system";
}

function writeStoredPreference(pref: ThemePreference) {
  try {
    localStorage.setItem(STORAGE_KEY, pref);
  } catch {
    // Ignore write failures
  }
}

// Shared singleton state so all components share the same reactive refs
const preference = ref<ThemePreference>(readStoredPreference());
const systemDark = ref(window.matchMedia("(prefers-color-scheme: dark)").matches);

// Listen for system preference changes (once, at module level)
const mql = window.matchMedia("(prefers-color-scheme: dark)");
function onSystemChange(e: MediaQueryListEvent) {
  systemDark.value = e.matches;
}
mql.addEventListener("change", onSystemChange);

// Clean up listener during Vite HMR to prevent duplicates
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    mql.removeEventListener("change", onSystemChange);
  });
}

// Module-level effect (runs once, not per-component)
let effectInitialized = false;

export function useTheme() {
  const isDark = computed(() =>
    preference.value === "system" ? systemDark.value : preference.value === "dark",
  );

  const monacoTheme = computed(() => (isDark.value ? "vs-dark" : "vs"));

  // Sync .dark class on <html> and persist preference (singleton)
  if (!effectInitialized) {
    effectInitialized = true;
    watchEffect(() => {
      document.documentElement.classList.toggle("dark", isDark.value);
      writeStoredPreference(preference.value);
    });
  }

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
