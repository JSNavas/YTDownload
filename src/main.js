const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const openDialog = window.__TAURI__.dialog?.open ?? null;

const $ = (id) => document.getElementById(id);
const el = (tag, cls) => { const n = document.createElement(tag); if (cls) n.className = cls; return n; };

/* ───────────────────────────────────────────────────────────────── i18n */

const I18N = {
  es: {
    pasteLabel: "Pega la URL de un video o de una playlist",
    analyze: "Analizar", analyzing: "Analizando…",
    clearSearch: "Limpiar la búsqueda",
    pasteFirst: "Pega una URL primero.",
    useSubfolder: "Crear una subcarpeta con el nombre de la playlist",
    whatToDownload: "Qué descargar",
    onlyThis: "Solo este video",
    wholeList: (n) => `Toda la playlist (${n})`,
    format: "Formato", fmtMp3: "MP3 (solo audio)", fmtVideo: "Video (MP4)",
    qAudio: "Calidad del audio", qVideo: "Calidad del video",
    saveTo: "Dónde se guarda", change: "Elegir…", finalPath: "Ruta final:",
    addToQueue: "Añadir a la cola", videoOne: "Video",
    playlistN: (n) => `Playlist · ${n} videos`,
    tabAll: "Todas", tabQueued: "En cola", tabDownloading: "Descargando",
    tabCompleted: "Completadas", tabFailed: "Fallidas", tabLater: "Pendientes",
    addLater: "Descargar más tarde", downloadNow: "Descargar ahora",
    clearFailed: "Quitar fallidas", stLater: "Pendiente",
    tLater: "Guardado para más tarde",
    tLaterN: (n) => `${n} videos guardados para más tarde`,
    aDownloadNow: "Descargar ahora",
    selectAll: "Seleccionar todas", startAll: "Descargar",
    retryFailed: "Reintentar fallidas", clearCompleted: "Quitar completadas",
    removeSelected: "Quitar seleccionadas", cancelAll: "Cancelar todo",
    nothingHere: "No hay nada aquí.",
    noDownloads: "Todavía no hay descargas. Pega una URL arriba para empezar.",
    settings: "Ajustes", language: "Idioma",
    simultaneous: "Descargas simultáneas",
    simultaneousHint: "Más no siempre es más rápido: YouTube limita las conexiones.",
    defaultFolderAudio: "Carpeta para la música",
    defaultFolderVideo: "Carpeta para los vídeos",
    defaultFolderHint: "Cada formato va a la suya, así la música y los vídeos no se mezclan.",
    history: "Historial",
    historyHint: "La cola se guarda en disco: al reabrir la app sigue estando.",
    clearHistory: "Borrar el historial",
    historyCleared: "Historial borrado",
    aboutLine: "Sin anuncios, sin límites, sin telemetría. Usa yt-dlp y ffmpeg.",
    stQueued: "En cola", stDownloading: "Descargando…", stConverting: "Convirtiendo…",
    stDone: "Listo", stFailed: "Falló", stCancelled: "Cancelado", stSkipped: "Ya estaba",
    ofN: (a, b) => `${a} de ${b}`,
    tOk: "Descargado", tFail: "Falló", tAdded: "Añadido a la cola",
    tAddedN: (n) => `${n} videos añadidos a la cola`,
    allDup: "Eso ya está en la lista.",
    dupSkipped: (n) => `${n} repetidas no se añadieron`,
    tAllDone: "Cola terminada", tAllDoneN: (ok, bad) =>
      bad ? `${ok} completadas, ${bad} fallidas` : `${ok} completadas`,
    aRetry: "Reintentar", aFolder: "Ver en carpeta", aRemove: "Quitar de la lista",
    aCancel: "Cancelar", missing: (w) => `Falta ${w} · instálalo con: brew install ${w}`,
    updAvail: (v) => `Nueva versión de yt-dlp (${v})`, update: "Actualizar",
    appUpdAvail: (v) => `YTDownload ${v} disponible`,
    appUpdGet: "Descargar",
    version: "Versión",
    checkUpdates: "Buscar actualizaciones",
    checking: "Buscando…",
    upToDate: (v) => `Estás en la última versión (${v})`,
    updFound: (v) => `Hay una versión nueva: ${v}`,
    updFailed: "No se pudo comprobar. ¿Hay conexión?",
    updating: "Actualizando…",
  },
  en: {
    pasteLabel: "Paste a video or playlist URL",
    analyze: "Analyse", analyzing: "Analysing…",
    clearSearch: "Clear the search",
    pasteFirst: "Paste a URL first.",
    useSubfolder: "Create a subfolder named after the playlist",
    whatToDownload: "What to download",
    onlyThis: "This video only",
    wholeList: (n) => `The whole playlist (${n})`,
    format: "Format", fmtMp3: "MP3 (audio only)", fmtVideo: "Video (MP4)",
    qAudio: "Audio quality", qVideo: "Video quality",
    saveTo: "Save to", change: "Change…", finalPath: "Final path:",
    addToQueue: "Add to queue", videoOne: "Video",
    playlistN: (n) => `Playlist · ${n} videos`,
    tabAll: "All", tabQueued: "Queued", tabDownloading: "Downloading",
    tabCompleted: "Completed", tabFailed: "Failed", tabLater: "Pending",
    addLater: "Download later", downloadNow: "Download now",
    clearFailed: "Clear failed", stLater: "Pending",
    tLater: "Saved for later",
    tLaterN: (n) => `${n} videos saved for later`,
    aDownloadNow: "Download now",
    selectAll: "Select all", startAll: "Download",
    retryFailed: "Retry failed", clearCompleted: "Clear completed",
    removeSelected: "Remove selected", cancelAll: "Cancel all",
    nothingHere: "Nothing here.",
    noDownloads: "No downloads yet. Paste a URL above to get started.",
    settings: "Settings", language: "Language",
    simultaneous: "Simultaneous downloads",
    simultaneousHint: "More isn't always faster: YouTube throttles connections.",
    defaultFolderAudio: "Folder for music",
    defaultFolderVideo: "Folder for videos",
    defaultFolderHint: "Each format has its own, so music and video don't end up mixed together.",
    history: "History",
    historyHint: "The queue is saved to disk, so it's still here next time you open the app.",
    clearHistory: "Clear the history",
    historyCleared: "History cleared",
    aboutLine: "No ads, no limits, no telemetry. Powered by yt-dlp and ffmpeg.",
    stQueued: "Queued", stDownloading: "Downloading…", stConverting: "Converting…",
    stDone: "Done", stFailed: "Failed", stCancelled: "Cancelled", stSkipped: "Already there",
    ofN: (a, b) => `${a} of ${b}`,
    tOk: "Downloaded", tFail: "Failed", tAdded: "Added to queue",
    tAddedN: (n) => `${n} videos added to the queue`,
    allDup: "That's already in the list.",
    dupSkipped: (n) => `${n} duplicates were skipped`,
    tAllDone: "Queue finished", tAllDoneN: (ok, bad) =>
      bad ? `${ok} completed, ${bad} failed` : `${ok} completed`,
    aRetry: "Retry", aFolder: "Show in folder", aRemove: "Remove from list",
    aCancel: "Cancel", missing: (w) => `${w} is missing · install it: brew install ${w}`,
    updAvail: (v) => `New yt-dlp version (${v})`, update: "Update",
    appUpdAvail: (v) => `YTDownload ${v} available`,
    appUpdGet: "Get it",
    version: "Version",
    checkUpdates: "Check for updates",
    checking: "Checking…",
    upToDate: (v) => `You are on the latest version (${v})`,
    updFound: (v) => `A new version is available: ${v}`,
    updFailed: "Could not check. Is there a connection?",
    updating: "Updating…",
  },
};
const t = (k, ...a) => {
  const v = I18N[S.lang]?.[k] ?? I18N.es[k] ?? k;
  return typeof v === "function" ? v(...a) : v;
};

/* ────────────────────────────────────────────────────────────── estado */

const S = {
  lang: "es",
  concurrency: 3,
  // Una carpeta por formato: mezclarlas es justo lo que no queremos.
  destMp3: "",
  destVideo: "",
  filter: "all",
  queue: [],
  running: new Set(),
  probe: null,
  destTouched: false,
  seq: 0,
  // Contadores de la TANDA en curso. No se pueden sacar de S.queue porque
  // ahí también está el historial de sesiones anteriores: contarlo diría
  // "4 completadas" cuando solo acabas de bajar una.
  batchActive: false,
  batchOk: 0,
  batchFail: 0,
  // Marca el turno en que cada item empezó a bajar en esta tanda. Se
  // conserva al terminar, y por eso la fila no salta al fondo.
  batchSeq: 0,
  // Cuántas filas hay pintadas. Una playlist de 300 no se dibuja entera:
  // se van añadiendo al bajar.
  limit: 10,
};
const PAGE = 10;

// localStorage puede fallar o venir vacío (ventana privada, datos borrados):
// la app tiene que funcionar igual, así que todo va envuelto.
function loadSettings() {
  try {
    const raw = localStorage.getItem("ytdownload.settings");
    if (raw) Object.assign(S, JSON.parse(raw));
  } catch (_) { /* valores por defecto */ }
  if (!I18N[S.lang]) S.lang = (navigator.language || "es").startsWith("en") ? "en" : "es";
  S.concurrency = Math.min(10, Math.max(1, parseInt(S.concurrency, 10) || 3));
}
function saveSettings() {
  try {
    localStorage.setItem("ytdownload.settings", JSON.stringify({
      lang: S.lang, concurrency: S.concurrency,
      destMp3: S.destMp3, destVideo: S.destVideo,
    }));
  } catch (_) { /* no es crítico */ }
}

const QUALITIES = {
  mp3: [["320", "320 kbps"], ["256", "256 kbps"], ["192", "192 kbps"],
        ["128", "128 kbps"], ["best", "best"]],
  video: [["best", "best"], ["2160", "4K · 2160p"], ["1440", "2K · 1440p"],
          ["1080", "Full HD · 1080p"], ["720", "HD · 720p"], ["480", "480p"]],
};
const DEFAULT_Q = { mp3: "192", video: "1080" };
const fmt = () => document.querySelector('input[name="fmt"]:checked').value;

// Una URL con v= y list= es ambigua. `scope` dice qué eligió el usuario;
// para el resto de URLs se deduce del tipo.
function scope() {
  if (!S.probe) return "video";
  if (S.probe.kind !== "both") return S.probe.kind;
  return document.querySelector('input[name="scope"]:checked').value;
}

function human(sec) {
  if (!sec || sec < 0) return "";
  const s = Math.round(sec), h = Math.floor(s / 3600),
        m = Math.floor((s % 3600) / 60), r = s % 60;
  return h ? `${h}:${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}`
           : `${m}:${String(r).padStart(2, "0")}`;
}
function setMsg(node, text, cls = "") {
  node.textContent = text;
  node.className = "msg" + (cls ? " " + cls : "");
}

/* ────────────────────────────────────────────────────────────── toasts */

const ICONS = {
  ok:   '<path d="M20 6 9 17l-5-5"/>',
  err:  '<circle cx="12" cy="12" r="9"/><path d="M12 7v6M12 16.5v.01"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8v.01"/>',
};
function toast(kind, title, text = "", ms = 4200, extra = "") {
  const node = el("div", `toast ${kind}${extra ? " " + extra : ""}`);
  node.innerHTML =
    `<svg class="ico" viewBox="0 0 24 24">${ICONS[kind] || ICONS.info}</svg>` +
    `<div class="toast-body"><div class="toast-title"></div>` +
    (text ? `<div class="toast-text"></div>` : "") + `</div>`;
  node.querySelector(".toast-title").textContent = title;
  if (text) node.querySelector(".toast-text").textContent = text;
  $("toasts").appendChild(node);
  // No dejamos que se acumulen sin fin durante una playlist larga.
  const all = $("toasts").children;
  while (all.length > 5) all[0].remove();
  setTimeout(() => {
    node.classList.add("out");
    setTimeout(() => node.remove(), 220);
  }, ms);
}

/* ────────────────────────────────────────────────────── i18n en el DOM */

function applyI18n() {
  document.documentElement.lang = S.lang;
  for (const node of document.querySelectorAll("[data-i18n]")) {
    node.textContent = t(node.dataset.i18n);
  }
  $("q-label").textContent = fmt() === "mp3" ? t("qAudio") : t("qVideo");
  $("probe").textContent = t("analyze");
  const cp = $("clear-probe");
  cp.title = t("clearSearch");
  cp.setAttribute("aria-label", t("clearSearch"));
  for (const r of document.querySelectorAll('input[name="lang"]')) r.checked = r.value === S.lang;
  if (S.probe) renderProbeHeader();
  render();
}

/* ─────────────────────────────────────────────────── herramientas / update */

async function refreshTools() {
  const box = $("tools");
  box.innerHTML = "";
  let st;
  try { st = await invoke("check_tools"); }
  catch (e) { box.innerHTML = `<span class="bad">${String(e)}</span>`; return; }

  $("about-tools").textContent =
    `yt-dlp ${st.ytdlp_version || "?"} · ffmpeg ${st.ffmpeg ? "ok" : "—"}`;

  const missing = [];
  if (!st.ytdlp) missing.push("yt-dlp");
  if (!st.ffmpeg) missing.push("ffmpeg");
  if (missing.length) {
    box.innerHTML = `<span class="bad"></span>`;
    box.firstChild.textContent = t("missing", missing.join(" "));
    return;
  }

  // Primero lo más importante: si hay una versión nueva de la propia app.
  invoke("check_app_update").then((info) => {
    if (!info) return;
    const span = el("span");
    span.textContent = t("appUpdAvail", info.version);
    const b = el("button", "btn sm primary");
    b.textContent = t("appUpdGet");
    b.onclick = () => invoke("open_url", { url: info.url }).catch(() => {});
    box.prepend(b);
    box.prepend(span);
  }).catch(() => {});

  // Y después, si hace falta actualizar yt-dlp.
  invoke("check_update").then((v) => {
    if (!v) return;
    const span = el("span");
    span.textContent = t("updAvail", v);
    const b = el("button", "btn sm");
    b.textContent = t("update");
    b.onclick = async () => {
      b.disabled = true; b.textContent = t("updating");
      try { toast("ok", await invoke("update_ytdlp")); }
      catch (e) { toast("err", String(e)); }
      refreshTools();
    };
    box.append(span, b);
  }).catch(() => {});
}

/* ──────────────────────────────────────────────────────── analizar URL */

function fillQuality() {
  const f = fmt(), sel = $("quality");
  sel.innerHTML = "";
  for (const [v, label] of QUALITIES[f]) {
    const o = el("option");
    o.value = v; o.textContent = label;
    if (v === DEFAULT_Q[f]) o.selected = true;
    sel.appendChild(o);
  }
  $("q-label").textContent = f === "mp3" ? t("qAudio") : t("qVideo");
}

const destKey = () => (fmt() === "mp3" ? "destMp3" : "destVideo");

async function refreshDest() {
  if (!S.destTouched) {
    $("dest").value = S[destKey()] || await invoke("default_dest", { format: fmt() });
  }
  await refreshTarget();
}

async function refreshTarget() {
  if (!S.probe) return;
  const useSub = scope() === "playlist" && $("use-folder").checked;
  $("target").textContent = await invoke("resolve_target", {
    dest: $("dest").value,
    folder: useSub ? ($("folder").value || S.probe.folder) : null,
    isPlaylist: useSub,
  });
}

function renderProbeHeader() {
  const p = S.probe, isPl = scope() === "playlist";
  // Con kind "both" la cabecera muestra lo que el usuario ha elegido.
  const v = p.kind === "both" && !isPl ? p.video : null;
  $("kind").textContent = isPl ? t("playlistN", p.count) : t("videoOne");
  $("title").textContent = v ? v.title : p.title;
  const dur = v ? v.duration : (!isPl ? p.duration : null);
  $("meta").textContent = [p.uploader, dur ? human(dur) : null].filter(Boolean).join(" · ");
  const th = v ? v.thumbnail : p.thumbnail;
  if (th) { $("thumb").src = th; $("thumb").parentElement.classList.remove("hidden"); }
  else $("thumb").parentElement.classList.add("hidden");
  $("folder-block").classList.toggle("hidden", !isPl);
}

async function doProbe() {
  const url = $("url").value.trim();
  if (!url) { setMsg($("probe-msg"), t("pasteFirst"), "err"); return; }
  $("probe").disabled = true;
  setMsg($("probe-msg"), t("analyzing"));
  try {
    S.probe = await invoke("probe_url", { url });
  } catch (e) {
    setMsg($("probe-msg"), String(e), "err");
    $("probe").disabled = false;
    toast("err", t("tFail"), String(e));
    return;
  }
  $("probe").disabled = false;
  setMsg($("probe-msg"), "");

  const both = S.probe.kind === "both";
  $("scope-block").classList.toggle("hidden", !both);
  if (both) {
    // Por defecto, solo el video: es lo que la URL señalaba.
    document.querySelector('input[name="scope"][value="video"]').checked = true;
    $("scope-video").textContent = t("onlyThis");
    $("scope-list").textContent = t("wholeList", S.probe.count);
  }
  if (S.probe.folder) $("folder").value = S.probe.folder || S.probe.title;
  renderProbeHeader();
  $("info").classList.remove("hidden");
  await refreshDest();
}

/* ───────────────────────────────────────────────────────────── la cola */

function thumbOf(url) {
  const m = url.match(/[?&]v=([\w-]{11})|youtu\.be\/([\w-]{11})/);
  const id = m ? (m[1] || m[2]) : null;
  return id ? `https://i.ytimg.com/vi/${id}/mqdefault.jpg` : "";
}

async function addToQueue(initialStatus = "queued") {
  const p = S.probe;
  if (!p) return;
  const later = initialStatus === "later";
  const isPl = scope() === "playlist";
  const useSub = isPl && $("use-folder").checked;
  const dest = await invoke("resolve_target", {
    dest: $("dest").value,
    folder: useSub ? ($("folder").value || p.folder) : null,
    isPlaylist: useSub,
  });
  const f = fmt(), q = $("quality").value;

  const one = p.kind === "both" ? p.video : null;
  const sources = isPl
    ? p.entries.map((e) => ({ url: e.url, title: e.title, index: e.index, duration: e.duration }))
    : [{
        // Con "both" usamos la URL limpia del video, sin el &list=,
        // para que yt-dlp no pueda arrastrar la lista entera.
        url: one ? one.url : $("url").value.trim(),
        title: one ? one.title : p.title,
        index: null,
        duration: one ? one.duration : p.duration,
      }];

  // Las listas "Mix" de YouTube repiten canciones, y volver a pegar la
  // misma URL también duplicaría. Una URL ya encolada no vuelve a entrar.
  // La repetición se mide por URL **y carpeta**: la misma canción puede
  // estar en dos playlists que van a destinos distintos, y en ese caso hay
  // que bajarla en cada una.
  const key = (url) => `${url}\u0000${dest}`;
  const known = new Set(S.queue.map((i) => `${i.url}\u0000${i.dest}`));
  const fresh = [];
  for (const s of sources) {
    if (known.has(key(s.url))) continue;
    known.add(key(s.url));
    fresh.push(s);
  }
  const skipped = sources.length - fresh.length;

  for (const s of fresh) {
    S.queue.push({
      id: `i${++S.seq}`,
      url: s.url, title: s.title, index: s.index, duration: s.duration,
      uploader: p.uploader || "", thumb: thumbOf(s.url),
      fmt: f, quality: q, dest,
      status: initialStatus, percent: 0, speed: "", eta: "",
      error: "", filepath: "", selected: false, batchSeq: 0,
    });
  }

  if (fresh.length === 0) {
    toast("info", t("allDup"));
    return;
  }

  if (fresh.length > 1) {
    toast("info", later ? t("tLaterN", fresh.length) : t("tAddedN", fresh.length), p.title);
  } else if (later) {
    toast("info", t("tLater"), p.title);
  }
  if (skipped > 0) toast("info", t("dupSkipped", skipped));
  render();
  persist();
  if (!later) pump();
  // Ya está en la lista: el panel de arriba ha cumplido su función y deja
  // el campo listo para la siguiente URL.
  clearProbe();
}

const byStatus = (st) => S.queue.filter((i) => i.status === st);

// Lo que está pasando ahora va arriba; el historial, al fondo.
// sort() es estable, así que dentro de cada grupo se respeta el orden
// de la playlist.
// 0 bajando · 1 recién terminadas (esta tanda) · 2 en cola · 3 fallos
// antiguos · 4 historial. Lo que acaba de terminar se queda arriba, pegado
// a las que siguen bajando.
function rankOf(i) {
  if (i.status === "downloading") return 0;
  const fresh = i.batchSeq > 0;
  if (fresh && (i.status === "done" || i.status === "failed")) return 1;
  if (i.status === "queued") return 2;
  if (i.status === "later") return 3;
  if (i.status === "failed" || i.status === "cancelled") return 4;
  return 5;
}

// Observa el final de la lista: cuando asoma, pinta otras PAGE filas.
let io = null;
function observeTail(node) {
  if (!io) {
    io = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        S.limit += PAGE;
        render();
      }
    }, { rootMargin: "240px" });
  }
  io.observe(node);
}
function visibleItems() {
  const base = S.filter === "all" ? S.queue : byStatus(S.filter);
  return [...base].sort((a, b) => {
    const d = rankOf(a) - rankOf(b);
    if (d !== 0) return d;
    // Entre las recién terminadas, la última arriba.
    if (rankOf(a) === 1) return (b.batchSeq || 0) - (a.batchSeq || 0);
    return 0;
  });
}

// Guardado con retardo: durante una playlist el estado cambia muchas veces
// seguidas y no tiene sentido escribir el archivo en cada una.
let persistTimer = null;
function persist() {
  clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    const items = S.queue.map(({ speed, eta, phase, selected, ...rest }) => rest);
    invoke("save_history", { items }).catch(() => {});
  }, 500);
}

// La carpeta por defecto no se queda vacía esperando a que el usuario la
// elija: se rellena con la del sistema en el primer arranque.
async function ensureDefaultDest() {
  try {
    if (!S.destMp3) S.destMp3 = await invoke("default_dest", { format: "mp3" });
    if (!S.destVideo) S.destVideo = await invoke("default_dest", { format: "video" });
    saveSettings();
  } catch (_) { /* refreshDest lo resolverá igualmente */ }
}

async function loadHistory() {
  let items;
  try { items = await invoke("load_history"); } catch (_) { return; }
  if (!Array.isArray(items) || items.length === 0) return;
  S.queue = items.map((i) => ({
    ...i,
    speed: "", eta: "", selected: false,
    // Lo que quedó a medias la última vez vuelve a la cola, no se reanuda
    // solo: que el usuario decida cuándo.
    batchSeq: 0,
    status: i.status === "downloading" ? "queued" : i.status,
    percent: i.status === "done" ? 100 : 0,
  }));
  S.seq = S.queue.reduce((m, i) => Math.max(m, parseInt(String(i.id).slice(1), 10) || 0), 0);
  render();
}

function updateCounts() {
  $("c-all").textContent = S.queue.length;
  $("c-queued").textContent = byStatus("queued").length;
  $("c-downloading").textContent = byStatus("downloading").length;
  $("c-later").textContent = byStatus("later").length;
  $("c-done").textContent = byStatus("done").length;
  $("c-failed").textContent = byStatus("failed").length;

  const done = byStatus("done").length, failed = byStatus("failed").length;
  const total = S.queue.length;

  const pending = byStatus("queued").length;
  const active = S.running.size > 0 || pending > 0;
  const batchDone = S.batchOk + S.batchFail;
  const batchTotal = batchDone + S.running.size + pending;
  $("overall").classList.toggle("hidden", !active || batchTotal === 0);
  if (batchTotal) {
    $("overall-fill").style.width = `${(batchDone / batchTotal) * 100}%`;
    $("overall-txt").textContent = t("ofN", batchDone, batchTotal);
  }
  // Cada botón aparece solo cuando hay algo sobre lo que actuar. Un botón
  // apagado que nunca se enciende es ruido: mejor que no esté.
  const queued = byStatus("queued").length;
  const retryable = failed + byStatus("cancelled").length;
  const anySelected = S.queue.some((i) => i.selected);
  const show = (id, cond) => $(id).classList.toggle("hidden", !cond);
  const later = byStatus("later").length;
  const selLater = S.queue.filter((i) => i.selected && i.status === "later").length;
  const onFailedTab = S.filter === "failed";
  const onLaterTab = S.filter === "later";

  // En la pestaña de pendientes solo manda "Descargar ahora": el otro
  // botón es para la cola y poner los dos juntos solo confunde.
  show("start", queued > 0 && S.running.size === 0 && !onLaterTab);
  // En Pendientes el botón manda sobre lo seleccionado, o sobre todas.
  show("start-later", later > 0 && (onLaterTab || selLater > 0));
  show("retry-failed", retryable > 0 && !onLaterTab);
  // "Quitar completadas" no pinta nada en la pestaña de fallidas.
  show("clear-done", done > 0 && !onFailedTab && !onLaterTab);
  show("clear-failed", failed > 0 && onFailedTab);
  show("remove-sel", anySelected);
  show("cancel-all", S.running.size > 0);

  // Y la barra entera desaparece mientras no haya cola.
  const visible = visibleItems().length;
  show("sel-all-label", visible > 0);
  show("tb-sep", visible > 0);
  document.querySelector(".toolbar").classList.toggle("hidden", total === 0);
}

const ACT = {
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  retry:  '<path d="M3 12a9 9 0 1 0 2.6-6.4"/><path d="M3 4v5h5"/>',
  remove: '<path d="M18 6 6 18M6 6l12 12"/>',
  cancel: '<rect x="6" y="6" width="12" height="12" rx="2"/>',
  now:    '<path d="M12 4v12"/><path d="M7 11l5 5 5-5"/><path d="M5 20h14"/>',
};
function actionBtn(kind, title, extra = "") {
  const b = el("button", `icon-btn ${extra}`);
  b.title = title;
  b.setAttribute("aria-label", title);
  b.innerHTML = `<svg viewBox="0 0 24 24">${ACT[kind]}</svg>`;
  return b;
}

function stateText(item) {
  switch (item.status) {
    case "queued": return t("stQueued");
    case "later": return t("stLater");
    case "downloading":
      if (item.phase === "post") return t("stConverting");
      return `${item.percent.toFixed(0)}%` +
        (item.speed ? ` · ${item.speed}` : "") + (item.eta ? ` · ${item.eta}` : "");
    case "done": return item.skipped ? t("stSkipped") : t("stDone");
    case "failed": return item.error || t("stFailed");
    case "cancelled": return t("stCancelled");
    default: return "";
  }
}

function render() {
  const list = $("list");
  if (io) io.disconnect();
  list.innerHTML = "";
  const all = visibleItems();
  const items = all.slice(0, S.limit);

  for (const item of items) {
    const li = el("li", `qi ${item.status}` + (item.selected ? " sel" : ""));

    const check = el("label", "check");
    check.innerHTML =
      `<input type="checkbox" ${item.selected ? "checked" : ""}>` +
      `<span class="box"><svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-6"/></svg></span>`;
    check.querySelector("input").onchange = (e) => {
      item.selected = e.target.checked;
      li.classList.toggle("sel", item.selected);
      updateCounts();
    };

    const img = el("img", "qi-thumb");
    img.alt = ""; img.loading = "lazy";
    if (item.thumb) img.src = item.thumb;

    const main = el("div", "qi-main");
    const ttl = el("div", "qi-title");
    ttl.textContent = (item.index ? `${item.index}. ` : "") + item.title;
    ttl.title = item.title;
    const sub = el("div", "qi-sub");
    const c1 = el("span", "chip"); c1.textContent = item.fmt === "mp3" ? "MP3" : "MP4";
    const c2 = el("span", "chip");
    c2.textContent = item.quality === "best" ? "best"
      : item.fmt === "mp3" ? `${item.quality}k` : `${item.quality}p`;
    const stt = el("span", "qi-state"); stt.textContent = stateText(item);
    stt.title = item.error || "";
    sub.append(c1, c2, stt);
    if (item.duration) { const d = el("span", "qi-state"); d.textContent = human(item.duration); sub.append(d); }
    main.append(ttl, sub);

    if (item.status === "downloading") {
      const bar = el("div", "bar qi-bar" + (item.phase === "post" ? " indet" : ""));
      const fill = el("div");
      if (item.phase !== "post") fill.style.width = `${item.percent}%`;
      bar.appendChild(fill);
      main.appendChild(bar);
    }

    const acts = el("div", "qi-actions");
    if (item.status === "downloading") {
      const b = actionBtn("cancel", t("aCancel"), "danger-i");
      b.onclick = () => cancelItem(item);
      acts.appendChild(b);
    }
    if (item.status === "later") {
      const b = actionBtn("now", t("aDownloadNow"), "ok-i");
      b.onclick = () => { item.status = "queued"; render(); persist(); pump(); };
      acts.appendChild(b);
    }
    if (item.status === "failed" || item.status === "cancelled") {
      const b = actionBtn("retry", t("aRetry"), "ok-i");
      b.onclick = () => { item.status = "queued"; item.error = ""; render(); persist(); pump(); };
      acts.appendChild(b);
    }
    if (item.status === "done") {
      const b = actionBtn("folder", t("aFolder"));
      b.onclick = () => item.filepath
        ? invoke("reveal_file", { path: item.filepath }).catch(() => {})
        : invoke("open_folder", { path: item.dest }).catch(() => {});
      acts.appendChild(b);
    }
    // Quitar borra el item de la lista, nunca el archivo del disco.
    const rm = actionBtn("remove", t("aRemove"), "danger-i");
    rm.onclick = () => {
      if (item.status === "downloading") cancelItem(item);
      S.queue = S.queue.filter((x) => x.id !== item.id);
      render();
      persist();
    };
    acts.appendChild(rm);

    li.append(check, img, main, acts);
    list.appendChild(li);
  }

  // La cola se ve desde el arranque: las pestañas son parte de la app,
  // no algo que aparece después de buscar.
  // Centinela: al entrar en pantalla pide el siguiente bloque.
  if (all.length > items.length) {
    const more = el("li", "more");
    more.innerHTML = `<span class="spin"></span>`;
    more.append(document.createTextNode(` ${all.length - items.length}`));
    list.appendChild(more);
    observeTail(more);
  }

  $("empty").classList.toggle("hidden", all.length > 0);
  $("empty").textContent = S.queue.length === 0 ? t("noDownloads") : t("nothingHere");
  const sel = all.filter((i) => i.selected).length;
  $("sel-all").checked = all.length > 0 && sel === all.length;
  updateCounts();
}

/* ─────────────────────────────────────────────────────── motor de descarga */

listen("item", ({ payload: p }) => {
  const item = S.queue.find((i) => i.id === p.id);
  if (!item || item.status !== "downloading") return;
  item.phase = p.phase;
  if (p.phase === "progress") {
    item.percent = p.percent; item.speed = p.speed; item.eta = p.eta;
  }
  // Repintamos solo esta fila: con 10 descargas a la vez, re-renderizar
  // la lista entera en cada tick haría la interfaz inusable.
  patchRow(item);
});

function patchRow(item) {
  const idx = visibleItems().indexOf(item);
  if (idx < 0) return;
  const li = $("list").children[idx];
  if (!li) return;
  const st = li.querySelector(".qi-state");
  if (st) st.textContent = stateText(item);
  const bar = li.querySelector(".qi-bar");
  if (bar) {
    const post = item.phase === "post";
    bar.classList.toggle("indet", post);
    const fill = bar.firstElementChild;
    if (fill && !post) fill.style.width = `${item.percent}%`;
  }
}

function pump() {
  if (!S.batchActive && byStatus("queued").length > 0) {
    S.batchActive = true;
    S.batchOk = 0;
    S.batchFail = 0;
    S.batchSeq = 0;
    for (const i of S.queue) i.batchSeq = 0;
  }
  while (S.running.size < S.concurrency) {
    const next = S.queue.find((i) => i.status === "queued");
    if (!next) break;
    runItem(next);
  }
  updateCounts();
}

async function runItem(item) {
  item.status = "downloading";
  item.batchSeq = ++S.batchSeq;
  item.percent = 0; item.speed = ""; item.eta = ""; item.error = "";
  S.running.add(item.id);
  render();

  try {
    const path = await invoke("download_item", {
      opts: {
        id: item.id, url: item.url, format: item.fmt,
        quality: item.quality, dest: item.dest, index: item.index,
      },
    });
    item.filepath = path || "";
    item.skipped = !path;
    item.status = "done";
    item.percent = 100;
    S.batchOk++;
    toast("ok", t("tOk"), item.title, 4200, "item-toast");
  } catch (e) {
    const msg = String(e);
    if (msg.includes("CANCELLED")) {
      item.status = "cancelled";
    } else {
      item.status = "failed";
      item.error = msg.replace(/^Error:\s*/, "");
      S.batchFail++;
      toast("err", t("tFail"), item.title, 6000, "item-toast");
    }
  } finally {
    S.running.delete(item.id);
    render();
    persist();
    if (S.running.size === 0 && !S.queue.some((i) => i.status === "queued")) {
      const ok = S.batchOk, bad = S.batchFail;
      S.batchActive = false;
      if (bad > 0) toast("info", t("tAllDone"), t("tAllDoneN", ok, bad), 6000);
    }
    pump();
  }
}

function cancelItem(item) {
  invoke("cancel_item", { id: item.id }).catch(() => {});
}

/* ───────────────────────────────────────────────────────────── settings */

// La versión la da el binario: tenerla escrita en el HTML significaba
// mostrar siempre la misma por mucho que la app se actualizara.
async function showVersion() {
  try { $("app-ver").textContent = "v" + await invoke("app_version"); }
  catch (_) { $("app-ver").textContent = ""; }
}

async function doCheckUpdates() {
  const b = $("check-updates");
  const msg = $("upd-msg");
  b.disabled = true;
  msg.className = "hint";
  msg.textContent = t("checking");
  let info = null, failed = false;
  try { info = await invoke("check_app_update"); } catch (_) { failed = true; }
  b.disabled = false;

  if (failed) { msg.className = "hint"; msg.textContent = t("updFailed"); return; }
  if (!info) {
    const v = await invoke("app_version").catch(() => "");
    msg.textContent = t("upToDate", v);
    return;
  }
  msg.textContent = "";
  const span = el("span");
  span.textContent = t("updFound", info.version) + " ";
  const link = el("button", "btn sm primary");
  link.textContent = t("appUpdGet");
  link.onclick = () => invoke("open_url", { url: info.url }).catch(() => {});
  msg.append(span, link);
}

function openSettings() {
  $("conc").value = S.concurrency;
  $("conc-out").value = S.concurrency;
  $("def-dest-mp3").value = S.destMp3;
  $("def-dest-video").value = S.destVideo;
  $("settings").classList.remove("hidden");
}
function closeSettings() { $("settings").classList.add("hidden"); }

/* ──────────────────────────────────────────────────────────── wiring */

// Deja la parte de arriba como recién abierta. No toca la cola:
// lo ya encolado o descargado sigue donde está.
function clearProbe() {
  S.probe = null;
  S.destTouched = false;
  $("url").value = "";
  $("info").classList.add("hidden");
  $("scope-block").classList.add("hidden");
  $("folder-block").classList.add("hidden");
  $("use-folder").checked = false;
  $("folder").value = "";
  $("folder").disabled = true;
  $("thumb").removeAttribute("src");
  setMsg($("probe-msg"), "");
  $("url").focus();
}

$("clear-probe").onclick = clearProbe;
$("probe").onclick = doProbe;
$("url").onkeydown = (e) => { if (e.key === "Enter") doProbe(); };
$("go").onclick = () => addToQueue("queued");
$("go-later").onclick = () => addToQueue("later");

$("pick").onclick = async () => {
  if (!openDialog) { $("dest").focus(); return; }
  const dir = await openDialog({ directory: true, multiple: false, defaultPath: $("dest").value });
  if (dir) { S.destTouched = true; $("dest").value = dir; refreshTarget(); }
};
$("dest").oninput = () => { S.destTouched = true; refreshTarget(); };
$("folder").oninput = refreshTarget;
for (const r of document.querySelectorAll('input[name="scope"]')) {
  r.onchange = () => { renderProbeHeader(); refreshTarget(); };
}
$("use-folder").onchange = () => {
  $("folder").disabled = !$("use-folder").checked;
  refreshTarget();
};
for (const r of document.querySelectorAll('input[name="fmt"]')) {
  r.onchange = () => { S.destTouched = false; fillQuality(); refreshDest(); };
}

for (const tab of document.querySelectorAll(".tab")) {
  tab.onclick = () => {
    for (const x of document.querySelectorAll(".tab")) x.classList.remove("active");
    tab.classList.add("active");
    S.filter = tab.dataset.filter;
    S.limit = PAGE;
    render();
  };
}

$("sel-all").onchange = (e) => {
  for (const i of visibleItems()) i.selected = e.target.checked;
  render();
};
$("start").onclick = pump;
$("start-later").onclick = () => {
  // Si hay selección, solo esas; si no, todas las pendientes.
  const sel = S.queue.filter((i) => i.selected && i.status === "later");
  const target = sel.length ? sel : byStatus("later");
  for (const i of target) { i.status = "queued"; i.selected = false; }
  render(); persist(); pump();
};
$("clear-failed").onclick = () => {
  S.queue = S.queue.filter((i) => i.status !== "failed" && i.status !== "cancelled");
  render(); persist();
};
$("retry-failed").onclick = () => {
  for (const i of S.queue) if (i.status === "failed" || i.status === "cancelled") {
    i.status = "queued"; i.error = "";
  }
  render(); persist(); pump();
};
$("clear-done").onclick = () => {
  S.queue = S.queue.filter((i) => i.status !== "done");
  render();
  persist();
};
$("remove-sel").onclick = () => {
  for (const i of S.queue) if (i.selected && i.status === "downloading") cancelItem(i);
  S.queue = S.queue.filter((i) => !i.selected);
  render();
  persist();
};
$("cancel-all").onclick = () => {
  invoke("cancel_all").catch(() => {});
  for (const i of S.queue) if (i.status === "queued") i.status = "cancelled";
  render();
};

$("clear-history").onclick = async () => {
  // Borra la lista y el archivo. No toca ningún archivo descargado.
  for (const i of S.queue) if (i.status === "downloading") cancelItem(i);
  S.queue = [];
  render();
  try { await invoke("clear_history"); toast("ok", t("historyCleared")); }
  catch (e) { toast("err", String(e)); }
};

$("check-updates").onclick = doCheckUpdates;
$("open-settings").onclick = openSettings;
$("close-settings").onclick = closeSettings;
$("settings").onclick = (e) => { if (e.target === $("settings")) closeSettings(); };
document.addEventListener("keydown", (e) => {
  if (e.key !== "Escape") return;
  if (!$("settings").classList.contains("hidden")) { closeSettings(); return; }
  if (!$("info").classList.contains("hidden")) clearProbe();
});
$("conc").oninput = (e) => {
  S.concurrency = parseInt(e.target.value, 10);
  $("conc-out").value = S.concurrency;
  saveSettings();
  pump();
};
for (const [field, key] of [["def-dest-mp3", "destMp3"], ["def-dest-video", "destVideo"]]) {
  $(field).oninput = (e) => { S[key] = e.target.value.trim(); saveSettings(); refreshDest(); };
  $("pick-" + field.replace("def-dest-", "def-")).onclick = async () => {
    if (!openDialog) { $(field).focus(); return; }
    const dir = await openDialog({ directory: true, multiple: false, defaultPath: S[key] });
    if (dir) { S[key] = dir; $(field).value = dir; saveSettings(); refreshDest(); }
  };
}
for (const r of document.querySelectorAll('input[name="lang"]')) {
  r.onchange = () => { S.lang = r.value; saveSettings(); applyI18n(); };
}

/* ───────────────────────────────────────────────────────────── arranque */

loadSettings();
fillQuality();
applyI18n();
refreshTools();
showVersion();
ensureDefaultDest().then(refreshDest);
loadHistory();
