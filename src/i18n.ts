// Переклади інтерфейсу. uk/en — основні; решта зроблені мною й не вичитані
// носіями — якщо десь ріже вухо, правиться тут одним рядком.

export type Locale =
  | "uk" | "en" | "es" | "pt" | "de" | "fr" | "it" | "nl"
  | "pl" | "cs" | "sk" | "ro" | "hu" | "tr" | "ru" | "zh" | "ja";

export const LOCALES: { code: Locale; label: string }[] = [
  { code: "uk", label: "Українська" },
  { code: "en", label: "English" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "de", label: "Deutsch" },
  { code: "fr", label: "Français" },
  { code: "it", label: "Italiano" },
  { code: "nl", label: "Nederlands" },
  { code: "pl", label: "Polski" },
  { code: "cs", label: "Čeština" },
  { code: "sk", label: "Slovenčina" },
  { code: "ro", label: "Română" },
  { code: "hu", label: "Magyar" },
  { code: "tr", label: "Türkçe" },
  { code: "ru", label: "Русский" },
  { code: "zh", label: "简体中文" },
  { code: "ja", label: "日本語" },
];

export type Key =
  | "searchPlaceholder" | "searchAria" | "clearSearch"
  | "pinned" | "recent"
  | "emptyNoResults" | "emptyHistory"
  | "pin" | "unpin" | "preview" | "closePreview" | "closeHint"
  | "hintSelect" | "hintPaste" | "hintPin" | "hintPreview"
  | "settings" | "hotkey" | "pressCombo" | "keepRecent"
  | "windowSize" | "lockResize" | "language" | "pinnedHint";

type Dict = Record<Key, string>;

export const messages: Record<Locale, Dict> = {
  uk: {
    searchPlaceholder: "Пошук в історії", searchAria: "Пошук в історії буферу", clearSearch: "Очистити пошук",
    pinned: "Закріплені", recent: "Останні",
    emptyNoResults: "Нічого не знайдено", emptyHistory: "Історія порожня. Скопіюйте текст або зображення — воно з'явиться тут.",
    pin: "Закріпити", unpin: "Відкріпити", preview: "Переглянути оригінал", closePreview: "Закрити перегляд", closeHint: "Esc або клік щоб закрити",
    hintSelect: "вибір", hintPaste: "вставити", hintPin: "закріпити", hintPreview: "перегляд",
    settings: "Налаштування", hotkey: "Гаряча клавіша", pressCombo: "Натисніть комбінацію…", keepRecent: "Зберігати останніх",
    windowSize: "Розмір вікна", lockResize: "Заблокувати зміну розміру", language: "Мова",
    pinnedHint: "Закріплені зберігаються завжди й не враховуються в цьому ліміті.",
  },
  en: {
    searchPlaceholder: "Search history", searchAria: "Search clipboard history", clearSearch: "Clear search",
    pinned: "Pinned", recent: "Recent",
    emptyNoResults: "Nothing found", emptyHistory: "History is empty. Copy text or an image and it will appear here.",
    pin: "Pin", unpin: "Unpin", preview: "View original", closePreview: "Close preview", closeHint: "Esc or click to close",
    hintSelect: "select", hintPaste: "paste", hintPin: "pin", hintPreview: "preview",
    settings: "Settings", hotkey: "Hotkey", pressCombo: "Press a combination…", keepRecent: "Keep recent",
    windowSize: "Window size", lockResize: "Lock window size", language: "Language",
    pinnedHint: "Pinned items are kept forever and don't count toward this limit.",
  },
  es: {
    searchPlaceholder: "Buscar en el historial", searchAria: "Buscar en el historial del portapapeles", clearSearch: "Borrar búsqueda",
    pinned: "Fijados", recent: "Recientes",
    emptyNoResults: "No se encontró nada", emptyHistory: "El historial está vacío. Copia texto o una imagen y aparecerá aquí.",
    pin: "Fijar", unpin: "Quitar", preview: "Ver original", closePreview: "Cerrar vista", closeHint: "Esc o clic para cerrar",
    hintSelect: "seleccionar", hintPaste: "pegar", hintPin: "fijar", hintPreview: "vista",
    settings: "Ajustes", hotkey: "Atajo global", pressCombo: "Pulsa una combinación…", keepRecent: "Guardar recientes",
    windowSize: "Tamaño de ventana", lockResize: "Bloquear el tamaño", language: "Idioma",
    pinnedHint: "Los elementos fijados se guardan siempre y no cuentan para este límite.",
  },
  pt: {
    searchPlaceholder: "Pesquisar no histórico", searchAria: "Pesquisar no histórico da área de transferência", clearSearch: "Limpar pesquisa",
    pinned: "Fixados", recent: "Recentes",
    emptyNoResults: "Nada encontrado", emptyHistory: "O histórico está vazio. Copie texto ou uma imagem e aparecerá aqui.",
    pin: "Fixar", unpin: "Desafixar", preview: "Ver original", closePreview: "Fechar visualização", closeHint: "Esc ou clique para fechar",
    hintSelect: "selecionar", hintPaste: "colar", hintPin: "fixar", hintPreview: "ver",
    settings: "Definições", hotkey: "Atalho global", pressCombo: "Prima uma combinação…", keepRecent: "Guardar recentes",
    windowSize: "Tamanho da janela", lockResize: "Bloquear o tamanho", language: "Idioma",
    pinnedHint: "Os itens fixados são guardados sempre e não contam para este limite.",
  },
  de: {
    searchPlaceholder: "Verlauf durchsuchen", searchAria: "Zwischenablage-Verlauf durchsuchen", clearSearch: "Suche löschen",
    pinned: "Angeheftet", recent: "Zuletzt",
    emptyNoResults: "Nichts gefunden", emptyHistory: "Der Verlauf ist leer. Kopiere Text oder ein Bild — es erscheint hier.",
    pin: "Anheften", unpin: "Lösen", preview: "Original ansehen", closePreview: "Vorschau schließen", closeHint: "Esc oder Klick zum Schließen",
    hintSelect: "wählen", hintPaste: "einfügen", hintPin: "anheften", hintPreview: "Vorschau",
    settings: "Einstellungen", hotkey: "Tastenkürzel", pressCombo: "Kombination drücken…", keepRecent: "Zuletzt behalten",
    windowSize: "Fenstergröße", lockResize: "Größe sperren", language: "Sprache",
    pinnedHint: "Angeheftete Einträge bleiben dauerhaft und zählen nicht zu diesem Limit.",
  },
  fr: {
    searchPlaceholder: "Rechercher dans l'historique", searchAria: "Rechercher dans l'historique du presse-papiers", clearSearch: "Effacer la recherche",
    pinned: "Épinglés", recent: "Récents",
    emptyNoResults: "Aucun résultat", emptyHistory: "L'historique est vide. Copiez du texte ou une image, il apparaîtra ici.",
    pin: "Épingler", unpin: "Détacher", preview: "Voir l'original", closePreview: "Fermer l'aperçu", closeHint: "Échap ou clic pour fermer",
    hintSelect: "sélection", hintPaste: "coller", hintPin: "épingler", hintPreview: "aperçu",
    settings: "Réglages", hotkey: "Raccourci global", pressCombo: "Appuyez sur une combinaison…", keepRecent: "Conserver les récents",
    windowSize: "Taille de la fenêtre", lockResize: "Verrouiller la taille", language: "Langue",
    pinnedHint: "Les éléments épinglés sont conservés et ne comptent pas dans cette limite.",
  },
  it: {
    searchPlaceholder: "Cerca nella cronologia", searchAria: "Cerca nella cronologia degli appunti", clearSearch: "Cancella ricerca",
    pinned: "Fissati", recent: "Recenti",
    emptyNoResults: "Nessun risultato", emptyHistory: "La cronologia è vuota. Copia del testo o un'immagine e apparirà qui.",
    pin: "Fissa", unpin: "Rimuovi", preview: "Vedi originale", closePreview: "Chiudi anteprima", closeHint: "Esc o clic per chiudere",
    hintSelect: "seleziona", hintPaste: "incolla", hintPin: "fissa", hintPreview: "anteprima",
    settings: "Impostazioni", hotkey: "Scorciatoia globale", pressCombo: "Premi una combinazione…", keepRecent: "Conserva recenti",
    windowSize: "Dimensioni finestra", lockResize: "Blocca le dimensioni", language: "Lingua",
    pinnedHint: "Gli elementi fissati restano sempre e non rientrano in questo limite.",
  },
  nl: {
    searchPlaceholder: "Zoek in geschiedenis", searchAria: "Zoek in klembordgeschiedenis", clearSearch: "Zoekopdracht wissen",
    pinned: "Vastgezet", recent: "Recent",
    emptyNoResults: "Niets gevonden", emptyHistory: "De geschiedenis is leeg. Kopieer tekst of een afbeelding en die verschijnt hier.",
    pin: "Vastzetten", unpin: "Losmaken", preview: "Origineel bekijken", closePreview: "Voorvertoning sluiten", closeHint: "Esc of klik om te sluiten",
    hintSelect: "kiezen", hintPaste: "plakken", hintPin: "vastzetten", hintPreview: "bekijken",
    settings: "Instellingen", hotkey: "Sneltoets", pressCombo: "Druk een combinatie…", keepRecent: "Recente bewaren",
    windowSize: "Venstergrootte", lockResize: "Grootte vergrendelen", language: "Taal",
    pinnedHint: "Vastgezette items blijven altijd bewaard en tellen niet mee voor deze limiet.",
  },
  pl: {
    searchPlaceholder: "Szukaj w historii", searchAria: "Szukaj w historii schowka", clearSearch: "Wyczyść wyszukiwanie",
    pinned: "Przypięte", recent: "Ostatnie",
    emptyNoResults: "Nic nie znaleziono", emptyHistory: "Historia jest pusta. Skopiuj tekst lub obraz, a pojawi się tutaj.",
    pin: "Przypnij", unpin: "Odepnij", preview: "Zobacz oryginał", closePreview: "Zamknij podgląd", closeHint: "Esc lub kliknij, aby zamknąć",
    hintSelect: "wybór", hintPaste: "wklej", hintPin: "przypnij", hintPreview: "podgląd",
    settings: "Ustawienia", hotkey: "Skrót globalny", pressCombo: "Naciśnij kombinację…", keepRecent: "Przechowuj ostatnie",
    windowSize: "Rozmiar okna", lockResize: "Zablokuj rozmiar", language: "Język",
    pinnedHint: "Przypięte pozycje są przechowywane zawsze i nie liczą się do tego limitu.",
  },
  cs: {
    searchPlaceholder: "Hledat v historii", searchAria: "Hledat v historii schránky", clearSearch: "Vymazat hledání",
    pinned: "Připnuté", recent: "Nedávné",
    emptyNoResults: "Nic nenalezeno", emptyHistory: "Historie je prázdná. Zkopírujte text nebo obrázek a objeví se zde.",
    pin: "Připnout", unpin: "Odepnout", preview: "Zobrazit originál", closePreview: "Zavřít náhled", closeHint: "Esc nebo klik pro zavření",
    hintSelect: "výběr", hintPaste: "vložit", hintPin: "připnout", hintPreview: "náhled",
    settings: "Nastavení", hotkey: "Klávesová zkratka", pressCombo: "Stiskněte kombinaci…", keepRecent: "Uchovat nedávné",
    windowSize: "Velikost okna", lockResize: "Uzamknout velikost", language: "Jazyk",
    pinnedHint: "Připnuté položky se uchovávají trvale a nepočítají se do tohoto limitu.",
  },
  sk: {
    searchPlaceholder: "Hľadať v histórii", searchAria: "Hľadať v histórii schránky", clearSearch: "Vymazať hľadanie",
    pinned: "Pripnuté", recent: "Nedávne",
    emptyNoResults: "Nič sa nenašlo", emptyHistory: "História je prázdna. Skopírujte text alebo obrázok a objaví sa tu.",
    pin: "Pripnúť", unpin: "Odopnúť", preview: "Zobraziť originál", closePreview: "Zavrieť náhľad", closeHint: "Esc alebo klik na zavretie",
    hintSelect: "výber", hintPaste: "vložiť", hintPin: "pripnúť", hintPreview: "náhľad",
    settings: "Nastavenia", hotkey: "Klávesová skratka", pressCombo: "Stlačte kombináciu…", keepRecent: "Uchovať nedávne",
    windowSize: "Veľkosť okna", lockResize: "Uzamknúť veľkosť", language: "Jazyk",
    pinnedHint: "Pripnuté položky sa uchovávajú trvalo a nerátajú sa do tohto limitu.",
  },
  ro: {
    searchPlaceholder: "Caută în istoric", searchAria: "Caută în istoricul clipboardului", clearSearch: "Șterge căutarea",
    pinned: "Fixate", recent: "Recente",
    emptyNoResults: "Nu s-a găsit nimic", emptyHistory: "Istoricul este gol. Copiază text sau o imagine și va apărea aici.",
    pin: "Fixează", unpin: "Anulează fixarea", preview: "Vezi originalul", closePreview: "Închide previzualizarea", closeHint: "Esc sau clic pentru a închide",
    hintSelect: "selectare", hintPaste: "lipire", hintPin: "fixare", hintPreview: "previzualizare",
    settings: "Setări", hotkey: "Scurtătură globală", pressCombo: "Apasă o combinație…", keepRecent: "Păstrează recentele",
    windowSize: "Dimensiunea ferestrei", lockResize: "Blochează dimensiunea", language: "Limbă",
    pinnedHint: "Elementele fixate se păstrează mereu și nu intră în această limită.",
  },
  hu: {
    searchPlaceholder: "Keresés az előzményekben", searchAria: "Keresés a vágólap előzményeiben", clearSearch: "Keresés törlése",
    pinned: "Rögzített", recent: "Legutóbbi",
    emptyNoResults: "Nincs találat", emptyHistory: "Az előzmények üresek. Másolj szöveget vagy képet, és itt megjelenik.",
    pin: "Rögzítés", unpin: "Feloldás", preview: "Eredeti megtekintése", closePreview: "Előnézet bezárása", closeHint: "Esc vagy kattintás a bezáráshoz",
    hintSelect: "kiválasztás", hintPaste: "beillesztés", hintPin: "rögzítés", hintPreview: "előnézet",
    settings: "Beállítások", hotkey: "Gyorsbillentyű", pressCombo: "Nyomj le egy kombinációt…", keepRecent: "Legutóbbiak megtartása",
    windowSize: "Ablakméret", lockResize: "Méret zárolása", language: "Nyelv",
    pinnedHint: "A rögzített elemek mindig megmaradnak, és nem számítanak bele ebbe a korlátba.",
  },
  tr: {
    searchPlaceholder: "Geçmişte ara", searchAria: "Pano geçmişinde ara", clearSearch: "Aramayı temizle",
    pinned: "Sabitlenenler", recent: "Son kopyalananlar",
    emptyNoResults: "Sonuç yok", emptyHistory: "Geçmiş boş. Bir metin veya görsel kopyalayın, burada görünecek.",
    pin: "Sabitle", unpin: "Sabitlemeyi kaldır", preview: "Orijinali gör", closePreview: "Önizlemeyi kapat", closeHint: "Kapatmak için Esc veya tıklayın",
    hintSelect: "seç", hintPaste: "yapıştır", hintPin: "sabitle", hintPreview: "önizleme",
    settings: "Ayarlar", hotkey: "Kısayol tuşu", pressCombo: "Bir kombinasyona basın…", keepRecent: "Son kayıtları tut",
    windowSize: "Pencere boyutu", lockResize: "Boyutu kilitle", language: "Dil",
    pinnedHint: "Sabitlenen öğeler her zaman saklanır ve bu sınıra dahil değildir.",
  },
  ru: {
    searchPlaceholder: "Поиск в истории", searchAria: "Поиск в истории буфера обмена", clearSearch: "Очистить поиск",
    pinned: "Закреплённые", recent: "Последние",
    emptyNoResults: "Ничего не найдено", emptyHistory: "История пуста. Скопируйте текст или изображение — оно появится здесь.",
    pin: "Закрепить", unpin: "Открепить", preview: "Посмотреть оригинал", closePreview: "Закрыть просмотр", closeHint: "Esc или клик, чтобы закрыть",
    hintSelect: "выбор", hintPaste: "вставить", hintPin: "закрепить", hintPreview: "просмотр",
    settings: "Настройки", hotkey: "Горячая клавиша", pressCombo: "Нажмите комбинацию…", keepRecent: "Хранить последних",
    windowSize: "Размер окна", lockResize: "Заблокировать размер", language: "Язык",
    pinnedHint: "Закреплённые хранятся всегда и не учитываются в этом лимите.",
  },
  zh: {
    searchPlaceholder: "搜索历史记录", searchAria: "搜索剪贴板历史", clearSearch: "清除搜索",
    pinned: "已固定", recent: "最近",
    emptyNoResults: "未找到内容", emptyHistory: "历史记录为空。复制文本或图片后会显示在这里。",
    pin: "固定", unpin: "取消固定", preview: "查看原图", closePreview: "关闭预览", closeHint: "按 Esc 或点击关闭",
    hintSelect: "选择", hintPaste: "粘贴", hintPin: "固定", hintPreview: "预览",
    settings: "设置", hotkey: "全局快捷键", pressCombo: "请按下组合键…", keepRecent: "保留最近条数",
    windowSize: "窗口大小", lockResize: "锁定窗口大小", language: "语言",
    pinnedHint: "已固定的内容会一直保留，不计入此上限。",
  },
  ja: {
    searchPlaceholder: "履歴を検索", searchAria: "クリップボード履歴を検索", clearSearch: "検索をクリア",
    pinned: "ピン留め", recent: "最近",
    emptyNoResults: "見つかりません", emptyHistory: "履歴は空です。テキストか画像をコピーするとここに表示されます。",
    pin: "ピン留め", unpin: "ピン留めを解除", preview: "原寸で表示", closePreview: "プレビューを閉じる", closeHint: "Esc またはクリックで閉じる",
    hintSelect: "選択", hintPaste: "貼り付け", hintPin: "ピン留め", hintPreview: "プレビュー",
    settings: "設定", hotkey: "ショートカット", pressCombo: "キーの組み合わせを押してください…", keepRecent: "保持する件数",
    windowSize: "ウインドウサイズ", lockResize: "サイズを固定", language: "言語",
    pinnedHint: "ピン留めした項目は常に保持され、この上限には含まれません。",
  },
};

export function isLocale(value: string): value is Locale {
  return value in messages;
}
