<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { LOCALES, isLocale, messages, type Key, type Locale } from "./i18n";

type PreviewItem = {
  id: string;
  isPinned: boolean;
  kind: "text" | "image";
  text: string;
  thumbPath: string | null;
  fullPath: string | null;
};

type ConfigDto = {
  recentLimit: number;
  hotkey: string;
  windowWidth: number;
  windowHeight: number;
  resizable: boolean;
  locale: string;
  shortcuts: Record<string, string>;
};

const ACTION_PIN = "pin";
const ACTION_PREVIEW = "preview";

const items = ref<PreviewItem[]>([]);
const query = ref("");
const copiedText = ref<string | null>(null);
// чий вибір відновити після того, як watcher перестворить Entry з новим id
const pendingSelect = ref<string | null>(null);
const selectedId = ref<string | null>(null);
const recentLimit = ref(10);
const hotkey = ref("");
const locale = ref<Locale>("uk");
const winWidth = ref(640);
const winHeight = ref(460);
const resizable = ref(true);
const shortcuts = ref<Record<string, string>>({
  [ACTION_PIN]: "Mod+KeyP",
  [ACTION_PREVIEW]: "Space",
});
const capturingAction = ref<string | null>(null);

const t = (key: Key) => messages[locale.value][key];
const capturing = ref(false);
const settingsOpen = ref(false);
const lightbox = ref<PreviewItem | null>(null);
const error = ref<string | null>(null);

const searchEl = ref<HTMLInputElement | null>(null);
const listEl = ref<HTMLElement | null>(null);

let timer: number | undefined;
let errorTimer: number | undefined;
let copiedTimer: number | undefined;

// Rust-команди повертають Result<(), String>; при Err invoke кидає рядок.
function showError(e: unknown) {
  error.value = typeof e === "string" ? e : String(e);
  clearTimeout(errorTimer);
  errorTimer = window.setTimeout(() => (error.value = null), 5000);
}

async function loadConfig() {
  try {
    const cfg = await invoke<ConfigDto>("get_config");
    recentLimit.value = cfg.recentLimit;
    hotkey.value = cfg.hotkey;
    winWidth.value = Math.round(cfg.windowWidth);
    winHeight.value = Math.round(cfg.windowHeight);
    resizable.value = cfg.resizable;
    shortcuts.value = { ...shortcuts.value, ...cfg.shortcuts };
    if (isLocale(cfg.locale)) locale.value = cfg.locale;
  } catch (e) {
    showError(e);
  }
}

async function refresh() {
  try {
    items.value = await invoke<PreviewItem[]>("get_previews");
  } catch (e) {
    showError(e);
  }
}

// backend віддає [pinned..., recent(старі→нові)]:
// закріплені показуємо окремою секцією, в останніх — найновіше першим.
const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return items.value;
  return items.value.filter((i) => i.text.toLowerCase().includes(q));
});
const pinned = computed(() => filtered.value.filter((i) => i.isPinned));
const recent = computed(() =>
  filtered.value.filter((i) => !i.isPinned).slice().reverse(),
);

const groups = computed(() =>
  [
    { key: "pinned" as Key, items: pinned.value },
    { key: "recent" as Key, items: recent.value },
  ].filter((g) => g.items.length),
);

// плаский порядок рядків = порядок, у якому по них ходять стрілки
const shown = computed(() => [...pinned.value, ...recent.value]);

const selectedIndex = computed(() =>
  shown.value.findIndex((i) => i.id === selectedId.value),
);

// Після copy watcher бачить власний запис у буфер і перестворює Entry з НОВИМ id —
// тому вибір відновлюємо за вмістом, інакше він злітає на початок списку.
watch(shown, (list) => {
  if (!list.length) {
    selectedId.value = null;
    return;
  }
  if (pendingSelect.value !== null) {
    const moved = list.find((i) => i.text === pendingSelect.value);
    if (moved) {
      selectedId.value = moved.id;
      pendingSelect.value = null;
      return;
    }
  }
  if (!list.some((i) => i.id === selectedId.value)) {
    selectedId.value = list[0].id;
  }
});

const selectedIsImage = computed(
  () => shown.value[selectedIndex.value]?.kind === "image",
);

function shortcutFor(item: PreviewItem): string {
  const i = shown.value.indexOf(item);
  return i >= 0 && i < 9 ? "\u2318" + (i + 1) : "";
}

function assetUrl(path: string | null): string | undefined {
  return path ? convertFileSrc(path) : undefined;
}

function move(delta: number) {
  const list = shown.value;
  if (!list.length) return;
  const next = Math.min(
    list.length - 1,
    Math.max(0, (selectedIndex.value === -1 ? 0 : selectedIndex.value) + delta),
  );
  selectedId.value = list[next].id;
  nextTick(() => {
    listEl.value
      ?.querySelector<HTMLElement>('[data-sel="1"]')
      ?.scrollIntoView({ block: "nearest" });
  });
}

async function copyItem(item: PreviewItem) {
  selectedId.value = item.id;
  pendingSelect.value = item.text;
  try {
    await invoke("copy", { id: item.id });
    copiedText.value = item.text;
    clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => (copiedText.value = null), 900);
  } catch (e) {
    showError(e);
  }
}

async function togglePin(item: PreviewItem) {
  try {
    await invoke(item.isPinned ? "remove_pinned" : "pin", { id: item.id });
    await refresh();
  } catch (e) {
    showError(e);
  }
}

async function applyLimit() {
  const limit = Math.min(100, Math.max(1, Math.round(recentLimit.value) || 1));
  recentLimit.value = limit;
  try {
    await invoke("set_recent_limit", { limit });
    await refresh();
  } catch (e) {
    showError(e);
  }
}

// Збирає комбінацію у форматі, який розуміє Tauri: "CmdOrCtrl+Shift+KeyV".
const MODIFIER_CODES = ["Meta", "Control", "Alt", "Shift"];

async function captureHotkey(e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();

  // самі модифікатори без основної клавіші комбінацією не є
  if (MODIFIER_CODES.some((m) => e.code.startsWith(m))) return;
  if (e.code === "Escape") {
    capturing.value = false;
    (e.target as HTMLElement)?.blur();
    return;
  }

  const parts: string[] = [];
  if (e.metaKey || e.ctrlKey) parts.push("CmdOrCtrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (parts.length === 0) return; // без модифікатора глобальний хоткей — погана ідея
  parts.push(e.code);

  const combination = parts.join("+");
  const previous = hotkey.value;
  hotkey.value = combination;
  capturing.value = false;
  (e.target as HTMLElement)?.blur();

  try {
    await invoke("set_hotkey", { hotkey: combination });
  } catch (err) {
    hotkey.value = previous; // backend відкотив реєстрацію — відкочуємо й показ
    showError(err);
  }
}

// "Mod+KeyP" / "Space" — формат локальних комбінацій (їх обробляє фронт)
function matchCombo(e: KeyboardEvent, combo: string | undefined): boolean {
  if (!combo) return false;
  const parts = combo.split("+");
  const code = parts[parts.length - 1];
  const mods = new Set(parts.slice(0, -1));
  if (e.code !== code) return false;
  if (mods.has("Mod") !== (e.metaKey || e.ctrlKey)) return false;
  if (mods.has("Alt") !== e.altKey) return false;
  if (mods.has("Shift") !== e.shiftKey) return false;
  return true;
}

function prettyCombo(combo: string | undefined): string {
  if (!combo) return "";
  return combo
    .split("+")
    .map((p) =>
      p === "Mod" ? "⌘" : p === "Shift" ? "⇧" : p === "Alt" ? "⌥"
        : p === "Space" ? "␣" : p.replace(/^Key|^Digit/, ""),
    )
    .join("");
}

async function captureAction(action: string, e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();
  if (MODIFIER_CODES.some((m) => e.code.startsWith(m))) return;
  if (e.code === "Escape") {
    capturingAction.value = null;
    (e.target as HTMLElement)?.blur();
    return;
  }

  const parts: string[] = [];
  if (e.metaKey || e.ctrlKey) parts.push("Mod");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(e.code);
  const combination = parts.join("+");

  const previous = shortcuts.value[action];
  shortcuts.value = { ...shortcuts.value, [action]: combination };
  capturingAction.value = null;
  (e.target as HTMLElement)?.blur();

  try {
    await invoke("set_shortcut", { action, combination });
  } catch (err) {
    shortcuts.value = { ...shortcuts.value, [action]: previous };
    showError(err);
  }
}

async function applyWindowSize() {
  const width = Math.min(2000, Math.max(420, Math.round(winWidth.value) || 640));
  const height = Math.min(1600, Math.max(320, Math.round(winHeight.value) || 460));
  winWidth.value = width;
  winHeight.value = height;
  try {
    await invoke("set_window_size", { width, height });
  } catch (e) {
    showError(e);
  }
}

async function applyResizable() {
  try {
    await invoke("set_resizable", { resizable: resizable.value });
  } catch (e) {
    showError(e);
  }
}

async function applyLocale() {
  try {
    await invoke("set_locale", { locale: locale.value });
  } catch (e) {
    showError(e);
  }
}

// "CmdOrCtrl+Shift+KeyV" -> "⌘⇧V"
const hotkeyPretty = computed(() => {
  if (!hotkey.value) return "";
  return hotkey.value
    .split("+")
    .map((p) =>
      p === "CmdOrCtrl"
        ? "⌘"
        : p === "Shift"
          ? "⇧"
          : p === "Alt"
            ? "⌥"
            : p.replace(/^Key|^Digit/, ""),
    )
    .join("");
});

const searchFocused = () => document.activeElement === searchEl.value;

function focusSearch() {
  searchEl.value?.focus();
}

/// WebView при показі панелі сам віддає фокус першому текстовому полю —
/// явно забираємо його на список, інакше Пробіл знову з'їдається пошуком.
function focusList() {
  if (searchFocused()) searchEl.value?.blur();
  listEl.value?.focus();
}

function onKeydown(e: KeyboardEvent) {
  if (capturing.value || capturingAction.value) return;

  if (lightbox.value) {
    if (e.key === "Escape" || matchCombo(e, shortcuts.value[ACTION_PREVIEW])) {
      e.preventDefault();
      lightbox.value = null;
    }
    return;
  }

  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
    e.preventDefault();
    focusSearch();
    return;
  }

  // Type-ahead: почав друкувати — пошук сам стає активним, символ потрапляє в поле.
  // Пробіл свідомо виключений: із нього не починають запит, тож він лишається
  // за переглядом оригіналу.
  if (
    !e.metaKey && !e.ctrlKey && !e.altKey &&
    e.key.length === 1 && e.key !== " " &&
    !searchFocused()
  ) {
    focusSearch();
    return; // без preventDefault — символ дійде до поля сам
  }

  // ⌘1…⌘9 — миттєва вставка перших дев'яти
  if ((e.metaKey || e.ctrlKey) && /^[1-9]$/.test(e.key)) {
    const item = shown.value[Number(e.key) - 1];
    if (item) {
      e.preventDefault();
      copyItem(item);
    }
    return;
  }

  if (matchCombo(e, shortcuts.value[ACTION_PIN])) {
    const item = shown.value[selectedIndex.value];
    if (item) {
      e.preventDefault();
      togglePin(item);
    }
    return;
  }

  if (matchCombo(e, shortcuts.value[ACTION_PREVIEW])) {
    const item = shown.value[selectedIndex.value];
    // якщо комбінація — голий Пробіл, у полі пошуку він має лишатись пробілом
    const bare = shortcuts.value[ACTION_PREVIEW] === "Space";
    if (item?.kind === "image" && !(bare && searchFocused())) {
      e.preventDefault();
      lightbox.value = item;
    }
    return;
  }

  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      move(1);
      break;
    case "ArrowUp":
      e.preventDefault();
      move(-1);
      break;
    case "Enter": {
      const item = shown.value[selectedIndex.value];
      if (item) {
        e.preventDefault();
        copyItem(item);
      }
      break;
    }
    case "Escape":
      e.preventDefault();
      if (settingsOpen.value) {
        settingsOpen.value = false;
      } else if (query.value) {
        query.value = "";
        searchEl.value?.blur(); // повертаємось у список
      } else if (searchFocused()) {
        searchEl.value?.blur();
      }
      break;
  }
}

function onWindowFocus() {
  // якщо людина нічого не шукала — повертаємось у список
  if (!query.value) nextTick(focusList);
}

onMounted(() => {
  loadConfig();
  refresh();
  nextTick(focusList);
  timer = window.setInterval(refresh, 1000); // бачити нові копії від watcher
  window.addEventListener("keydown", onKeydown);
  // панель ховається й показується без перемонтування — фокус треба забирати щоразу
  window.addEventListener("focus", onWindowFocus);
});

onUnmounted(() => {
  clearInterval(timer);
  clearTimeout(errorTimer);
  clearTimeout(copiedTimer);
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("focus", onWindowFocus);
});
</script>

<template>
  <main class="panel">
    <div class="search-row">
      <svg class="search-icon" viewBox="0 0 16 16" aria-hidden="true">
        <path
          d="M7 1.5a5.5 5.5 0 1 1-3.6 9.66l-2.2 2.2a.75.75 0 0 1-1.06-1.06l2.2-2.2A5.5 5.5 0 0 1 7 1.5Zm0 1.5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z"
          fill="currentColor"
        />
      </svg>
      <input
        ref="searchEl"
        v-model="query"
        class="search"
        type="search"
        :placeholder="t('searchPlaceholder')"
        :aria-label="t('searchAria')"
        spellcheck="false"
        autocomplete="off"
      />
      <button
        v-if="query"
        class="icon-btn clear"
        :aria-label="t('clearSearch')"
        @click="query = ''"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M8 1a7 7 0 1 1 0 14A7 7 0 0 1 8 1Zm2.47 4.47L8 7.94 5.53 5.47 4.47 6.53 6.94 9l-2.47 2.47 1.06 1.06L8 10.06l2.47 2.47 1.06-1.06L9.06 9l2.47-2.47-1.06-1.06Z"
            fill="currentColor"
          />
        </svg>
      </button>
    </div>

    <div ref="listEl" class="scroll" tabindex="-1">
      <template v-if="shown.length">
        <section v-for="group in groups" :key="group.key" class="group">
          <h2 class="group-title">{{ t(group.key) }}</h2>
          <ul class="list" role="listbox" :aria-label="t(group.key)">
            <li
              v-for="item in group.items"
              :key="item.id"
              class="row"
              role="option"
              :aria-selected="item.id === selectedId"
              :data-sel="item.id === selectedId ? '1' : '0'"
              :class="{
                sel: item.id === selectedId,
                copied: item.text === copiedText,
              }"
              @click="copyItem(item)"
            >
              <span class="rail">
                <span class="shortcut">{{ shortcutFor(item) }}</span>
              </span>
              <span class="body">
                <img
                  v-if="item.kind === 'image' && item.thumbPath"
                  class="thumb"
                  :src="assetUrl(item.thumbPath)"
                  :alt="item.text"
                  :title="item.text"
                />
                <span v-else class="text">{{ item.text }}</span>
              </span>
              <span class="tools">
                <button
                  v-if="item.kind === 'image'"
                  class="icon-btn peek"
                  :aria-label="t('preview')"
                  :title="`${t('preview')} (${prettyCombo(shortcuts[ACTION_PREVIEW])})`"
                  @click.stop="lightbox = item"
                >
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path
                      d="M8 3c3.5 0 6.2 2.4 7.2 4.6.2.3.2.6 0 .9C14.2 10.6 11.5 13 8 13S1.8 10.6.8 8.5a1 1 0 0 1 0-.9C1.8 5.4 4.5 3 8 3Zm0 1.6a3.4 3.4 0 1 0 0 6.8 3.4 3.4 0 0 0 0-6.8Zm0 1.6a1.8 1.8 0 1 1 0 3.6 1.8 1.8 0 0 1 0-3.6Z"
                      fill="currentColor"
                    />
                  </svg>
                </button>
                <button
                  class="icon-btn pin"
                  :class="{ on: item.isPinned }"
                  :aria-label="item.isPinned ? t('unpin') : t('pin')"
                  :title="`${item.isPinned ? t('unpin') : t('pin')} (${prettyCombo(shortcuts[ACTION_PIN])})`"
                  @click.stop="togglePin(item)"
                >
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path
                      d="m8 1.6 1.9 3.9 4.3.6-3.1 3 .7 4.3L8 11.4l-3.8 2 .7-4.3-3.1-3 4.3-.6L8 1.6Z"
                      :fill="item.isPinned ? 'currentColor' : 'none'"
                      stroke="currentColor"
                      stroke-width="1.3"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
              </span>
            </li>
          </ul>
        </section>
      </template>

      <p v-else class="empty">
        {{ query ? t("emptyNoResults") : t("emptyHistory") }}
      </p>
    </div>

    <footer class="status">
      <span class="status-text">
        <kbd v-if="hotkeyPretty">{{ hotkeyPretty }}</kbd>
        <span v-if="shown.length" class="hints">
          ↑↓ {{ t("hintSelect") }} · ⏎ {{ t("hintPaste") }} ·
          {{ prettyCombo(shortcuts[ACTION_PIN]) }} {{ t("hintPin") }}
          <template v-if="selectedIsImage">
            · {{ prettyCombo(shortcuts[ACTION_PREVIEW]) }} {{ t("hintPreview") }}
          </template>
        </span>
      </span>
      <button
        class="icon-btn gear"
        :class="{ active: settingsOpen }"
        :aria-label="t('settings')"
        :aria-expanded="settingsOpen"
        @click="settingsOpen = !settingsOpen"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M8 5.2a2.8 2.8 0 1 0 0 5.6 2.8 2.8 0 0 0 0-5.6Zm0 1.4a1.4 1.4 0 1 1 0 2.8 1.4 1.4 0 0 1 0-2.8Z"
            fill="currentColor"
          />
          <path
            d="M6.9.9h2.2l.3 1.7 1.2.5 1.4-1 1.6 1.6-1 1.4.5 1.2 1.7.3v2.2l-1.7.3-.5 1.2 1 1.4-1.6 1.6-1.4-1-1.2.5-.3 1.7H6.9l-.3-1.7-1.2-.5-1.4 1-1.6-1.6 1-1.4-.5-1.2L1.2 9.1V6.9l1.7-.3.5-1.2-1-1.4 1.6-1.6 1.4 1 1.2-.5.3-1.7Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </footer>

    <section v-if="settingsOpen" class="settings">
      <div class="field">
        <label for="locale">{{ t("language") }}</label>
        <select id="locale" v-model="locale" class="select" @change="applyLocale">
          <option v-for="l in LOCALES" :key="l.code" :value="l.code">
            {{ l.label }}
          </option>
        </select>
      </div>

      <div class="field">
        <label for="hotkey">{{ t("hotkey") }}</label>
        <input
          id="hotkey"
          class="hotkey"
          :class="{ capturing }"
          type="text"
          readonly
          :value="capturing ? t('pressCombo') : hotkeyPretty"
          @focus="capturing = true"
          @blur="capturing = false"
          @keydown="captureHotkey"
        />
      </div>

      <div class="field">
        <label for="pinKey">{{ t("pin") }}</label>
        <input
          id="pinKey"
          class="hotkey"
          :class="{ capturing: capturingAction === ACTION_PIN }"
          type="text"
          readonly
          :value="
            capturingAction === ACTION_PIN
              ? t('pressCombo')
              : prettyCombo(shortcuts[ACTION_PIN])
          "
          @focus="capturingAction = ACTION_PIN"
          @blur="capturingAction = null"
          @keydown="captureAction(ACTION_PIN, $event)"
        />
      </div>

      <div class="field">
        <label for="previewKey">{{ t("preview") }}</label>
        <input
          id="previewKey"
          class="hotkey"
          :class="{ capturing: capturingAction === ACTION_PREVIEW }"
          type="text"
          readonly
          :value="
            capturingAction === ACTION_PREVIEW
              ? t('pressCombo')
              : prettyCombo(shortcuts[ACTION_PREVIEW])
          "
          @focus="capturingAction = ACTION_PREVIEW"
          @blur="capturingAction = null"
          @keydown="captureAction(ACTION_PREVIEW, $event)"
        />
      </div>

      <div class="field">
        <label for="limit">{{ t("keepRecent") }}</label>
        <input
          id="limit"
          v-model.number="recentLimit"
          class="limit"
          type="number"
          min="1"
          max="100"
          @change="applyLimit"
        />
      </div>

      <div class="field">
        <label for="winW">{{ t("windowSize") }}</label>
        <span class="size">
          <input
            id="winW"
            v-model.number="winWidth"
            class="limit"
            type="number"
            min="420"
            max="2000"
            @change="applyWindowSize"
          />
          <span class="times">×</span>
          <input
            v-model.number="winHeight"
            class="limit"
            type="number"
            min="320"
            max="1600"
            aria-label="height"
            @change="applyWindowSize"
          />
        </span>
      </div>

      <div class="field">
        <label for="lockSize">{{ t("lockResize") }}</label>
        <input
          id="lockSize"
          :checked="!resizable"
          type="checkbox"
          @change="
            resizable = !($event.target as HTMLInputElement).checked;
            applyResizable();
          "
        />
      </div>

      <p class="hint">{{ t("pinnedHint") }}</p>
    </section>

    <div
      v-if="lightbox"
      class="lightbox"
      role="dialog"
      aria-modal="true"
      :aria-label="t('preview')"
      @click="lightbox = null"
    >
      <button class="lightbox-close" :aria-label="t('closePreview')">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M12.5 4.56 11.44 3.5 8 6.94 4.56 3.5 3.5 4.56 6.94 8 3.5 11.44l1.06 1.06L8 9.06l3.44 3.44 1.06-1.06L9.06 8l3.44-3.44Z"
            fill="currentColor"
          />
        </svg>
      </button>
      <img :src="assetUrl(lightbox.fullPath)" :alt="lightbox.text" />
      <p class="lightbox-meta">{{ lightbox.text }} · {{ t("closeHint") }}</p>
    </div>

    <p v-if="error" class="error" role="alert">{{ error }}</p>
  </main>
</template>

<style scoped>
.panel {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--surface);
  color: var(--content);
  overflow: hidden;
}

/* ---- пошук ---- */
.search-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--separator);
  flex: none;
}
.search-icon {
  width: 15px;
  height: 15px;
  flex: none;
  color: var(--content-secondary);
}
.search {
  flex: 1;
  min-width: 0;
  border: none;
  background: none;
  color: inherit;
  font: inherit;
  font-size: 15px;
  outline: none;
  padding: 2px 0;
}
.search::placeholder {
  color: var(--content-secondary);
}
.search::-webkit-search-cancel-button {
  display: none;
}

/* ---- список ---- */
.scroll {
  flex: 1;
  outline: none;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.group {
  padding-bottom: 4px;
}
.group-title {
  margin: 0;
  padding: 10px 14px 4px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--content-secondary);
}
.list {
  list-style: none;
  margin: 0;
  padding: 0 6px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 34px;
  padding: 4px 6px;
  border-radius: 7px;
  cursor: default;
}
.row:has(.thumb) {
  min-height: 72px;
}
.row:hover {
  background: var(--row-hover);
}
.row.sel,
.row.sel:hover {
  background: var(--accent);
  color: var(--on-accent);
}
.row.copied {
  background: var(--accent-strong);
  color: var(--on-accent);
}

/* рейка: або мініатюра, або номер для ⌘N */
.rail {
  flex: none;
  width: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.thumb {
  display: block;
  height: 64px;
  max-width: 100%;
  width: auto;
  object-fit: contain;
  border-radius: 5px;
  background: var(--thumb-bg);
  box-shadow: inset 0 0 0 1px var(--separator);
}
.shortcut {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--content-secondary);
}
.row.sel .shortcut,
.row.copied .shortcut {
  color: var(--on-accent-secondary);
}

.body {
  flex: 1;
  min-width: 0;
}
.text {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  font-size: 13px;
  line-height: 1.4;
  /* зберігаємо переноси з буфера: код і адреси так читабельніші */
  white-space: pre-wrap;
  word-break: break-word;
}

/* ---- кнопки-іконки ---- */
.tools {
  flex: none;
  display: flex;
  gap: 2px;
}
.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--content-secondary);
  cursor: default;
  padding: 0;
}
.icon-btn svg {
  width: 15px;
  height: 15px;
}
.icon-btn:hover {
  background: var(--control-hover);
  color: var(--content);
}
.icon-btn:focus-visible {
  outline: 2px solid var(--focus);
  outline-offset: -2px;
}
.row .icon-btn {
  opacity: 0;
}
.row:hover .icon-btn,
.row.sel .icon-btn,
.row .icon-btn:focus-visible,
.row .icon-btn.on,
.row .icon-btn.peek {
  opacity: 1;
}
.row.sel .icon-btn {
  color: var(--on-accent-secondary);
}
.row.sel .icon-btn:hover {
  background: rgba(255, 255, 255, 0.22);
  color: var(--on-accent);
}
.pin.on {
  color: var(--accent-text);
}
.row.sel .pin.on {
  color: var(--on-accent);
}

/* ---- порожньо ---- */
.empty {
  margin: 0;
  padding: 48px 32px;
  text-align: center;
  font-size: 13px;
  line-height: 1.5;
  color: var(--content-secondary);
}

/* ---- статус ---- */
.status {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px 6px 14px;
  border-top: 1px solid var(--separator);
  font-size: 11px;
  color: var(--content-secondary);
}
.status-text {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
}
kbd {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--control-hover);
  color: var(--content);
}
.gear.active {
  background: var(--control-hover);
  color: var(--content);
}

/* ---- налаштування ---- */
.settings {
  flex: none;
  padding: 12px 14px 14px;
  border-top: 1px solid var(--separator);
  background: var(--surface-raised);
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.field {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}
.field label {
  flex: 1;
  color: var(--content);
}
.hotkey,
.limit {
  border: 1px solid var(--field-border);
  border-radius: 6px;
  background: var(--field);
  color: inherit;
  font-size: 12px;
  padding: 4px 8px;
  outline: none;
}
.hotkey {
  width: 180px;
  text-align: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  cursor: default;
}
.hotkey.capturing {
  font-family: inherit;
  border-color: var(--accent);
  color: var(--accent-text);
}
.limit {
  width: 64px;
}
.select {
  border: 1px solid var(--field-border);
  border-radius: 6px;
  background: var(--field);
  color: inherit;
  font: inherit;
  font-size: 12px;
  padding: 3px 6px;
}
.size {
  display: flex;
  align-items: center;
  gap: 6px;
}
.times {
  color: var(--content-secondary);
}
.hints {
  overflow: hidden;
  text-overflow: ellipsis;
}
.hotkey:focus-visible,
.limit:focus-visible {
  border-color: var(--accent);
  outline: 2px solid var(--focus);
  outline-offset: 1px;
}
.hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  color: var(--content-secondary);
}

/* ---- лайтбокс ---- */
.lightbox {
  position: absolute;
  inset: 0;
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px;
  background: var(--scrim);
}
.lightbox-close {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.14);
  color: #f2f2f5;
  cursor: default;
  padding: 0;
}
.lightbox-close svg {
  width: 15px;
  height: 15px;
}
.lightbox-close:hover {
  background: rgba(255, 255, 255, 0.26);
}
.lightbox img {
  max-width: 100%;
  max-height: calc(100% - 32px);
  object-fit: contain;
  border-radius: 8px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.45);
}
.lightbox-meta {
  margin: 0;
  font-size: 11px;
  color: #f2f2f5;
}

/* ---- помилка ---- */
.error {
  position: absolute;
  left: 12px;
  right: 12px;
  bottom: 12px;
  z-index: 20;
  margin: 0;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--danger-line);
  background: var(--danger-bg);
  color: var(--danger);
  font-size: 12px;
  line-height: 1.4;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
}
</style>

<style>
:root {
  color-scheme: light dark;

  --surface: #ffffff;
  --surface-raised: #fbfbfd;
  --content: #1d1d1f; /* 16.8:1 на surface */
  --content-secondary: #62626b; /* 6.0:1 на surface */
  --separator: rgba(0, 0, 0, 0.1);
  --control-hover: rgba(0, 0, 0, 0.06);
  --row-hover: rgba(0, 0, 0, 0.035);
  --field: #ffffff;
  --field-border: rgba(0, 0, 0, 0.18);
  --thumb-bg: #f0f0f2;

  --accent: #0a69da; /* білий текст: 5.2:1 */
  --accent-strong: #0a5fc4;
  --accent-text: #0a5fc4; /* 6.1:1 на surface */
  --on-accent: #ffffff;
  --on-accent-secondary: rgba(255, 255, 255, 0.82);
  --focus: #0a69da;

  --danger: #b3261e; /* 5.7:1 на danger-bg */
  --danger-bg: #fdeceb;
  --danger-line: #f1c4c1;
  --scrim: rgba(20, 20, 22, 0.86);

  font-family:
    -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
  font-size: 13px;
  -webkit-font-smoothing: antialiased;
}

@media (prefers-color-scheme: dark) {
  :root {
    --surface: #262628;
    --surface-raised: #2b2b2e;
    --content: #f2f2f5; /* 13.5:1 */
    --content-secondary: #a2a2ab; /* 6.0:1 */
    --separator: rgba(255, 255, 255, 0.12);
    --control-hover: rgba(255, 255, 255, 0.1);
    --row-hover: rgba(255, 255, 255, 0.05);
    --field: #1c1c1e;
    --field-border: rgba(255, 255, 255, 0.2);
    --thumb-bg: #1c1c1e;

    --accent: #0a66d0; /* білий текст: 5.5:1 */
    --accent-strong: #0a74e8;
    --accent-text: #6cb4ff; /* 6.9:1 */
    --on-accent: #ffffff;
    --on-accent-secondary: rgba(255, 255, 255, 0.85);
    --focus: #6cb4ff;

    --danger: #ff9a92; /* 7.4:1 */
    --danger-bg: #3a1f1d;
    --danger-line: #5c2b27;
    --scrim: rgba(0, 0, 0, 0.88);
  }
}

html,
body {
  margin: 0;
  height: 100%;
}
#app {
  height: 100%;
}

@media (prefers-reduced-motion: reduce) {
  * {
    transition: none !important;
    animation: none !important;
  }
}
</style>
