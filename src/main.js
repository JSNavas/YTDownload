const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
// El plugin dialog se expone vía withGlobalTauri; si por lo que sea no está,
// no queremos que explote todo el script: caemos a escribir la ruta a mano.
const openDialog = window.__TAURI__.dialog?.open ?? null;

const $ = (id) => document.getElementById(id);

const QUALITIES = {
  mp3: [
    ["320", "320 kbps (máxima)"],
    ["256", "256 kbps"],
    ["192", "192 kbps (recomendada)"],
    ["128", "128 kbps (ligera)"],
    ["best", "La mejor que haya"],
  ],
  video: [
    ["best", "La mejor que haya"],
    ["2160", "4K · 2160p"],
    ["1440", "2K · 1440p"],
    ["1080", "Full HD · 1080p"],
    ["720", "HD · 720p"],
    ["480", "480p (ligera)"],
  ],
};
const DEFAULT_Q = { mp3: "192", video: "1080" };

let probe = null;        // resultado de analizar la URL
let items = new Map();   // index -> <li>
let destTouched = false; // si el usuario eligió carpeta a mano, no la sobreescribimos
let downloading = false;
let okCount = 0, failCount = 0;

const fmt = () => document.querySelector('input[name="fmt"]:checked').value;

function setMsg(el, text, cls = "") {
  el.textContent = text;
  el.className = "msg" + (cls ? " " + cls : "");
}

function human(sec) {
  if (!sec || sec < 0) return "";
  const s = Math.round(sec);
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), r = s % 60;
  return h ? `${h}h ${m}m` : m ? `${m}:${String(r).padStart(2, "0")}` : `0:${String(r).padStart(2, "0")}`;
}

// ------------------------------------------------------------- estado de tools

async function refreshTools() {
  const box = $("tools");
  box.innerHTML = "";
  let t;
  try {
    t = await invoke("check_tools");
  } catch (e) {
    box.innerHTML = `<span class="bad">${String(e)}</span>`;
    return;
  }

  // Falta algo imprescindible: eso sí hay que decirlo, y con el comando exacto.
  const faltan = [];
  if (!t.ytdlp) faltan.push("yt-dlp");
  if (!t.ffmpeg) faltan.push("ffmpeg");
  if (faltan.length) {
    box.innerHTML =
      `<span class="bad">Falta ${faltan.join(" y ")} · instálalo con: ` +
      `brew install ${faltan.join(" ")}</span>`;
    return;
  }

  // Todo correcto: la cabecera se queda vacía. Solo hablamos si hay novedad,
  // y eso se comprueba en segundo plano para no retrasar el arranque.
  invoke("check_update").then((nueva) => {
    if (!nueva) return;
    box.innerHTML =
      `<span>Hay una versión nueva de yt-dlp (${nueva})</span>` +
      ` <button id="upd" type="button">Actualizar</button>`;
    $("upd").onclick = async () => {
      const b = $("upd");
      b.disabled = true;
      b.textContent = "Actualizando…";
      try {
        const res = await invoke("update_ytdlp");
        setMsg($("probe-msg"), res, "ok");
      } catch (e) {
        setMsg($("probe-msg"), String(e), "err");
      }
      // Si fue bien, el aviso desaparece solo en este refresco.
      await refreshTools();
    };
  }).catch(() => {});
}

// --------------------------------------------------------------------- calidad

function fillQuality() {
  const f = fmt();
  const sel = $("quality");
  sel.innerHTML = "";
  for (const [v, label] of QUALITIES[f]) {
    const o = document.createElement("option");
    o.value = v; o.textContent = label;
    if (v === DEFAULT_Q[f]) o.selected = true;
    sel.appendChild(o);
  }
  $("q-label").textContent = f === "mp3" ? "Calidad del audio" : "Calidad del video";
}

async function refreshDest({ force = false } = {}) {
  if (force || !destTouched) {
    $("dest").value = await invoke("default_dest", { format: fmt() });
  }
  await refreshTarget();
}

async function refreshTarget() {
  if (!probe) return;
  const useSub = probe.kind === "playlist" && $("use-folder").checked;
  const t = await invoke("resolve_target", {
    dest: $("dest").value,
    folder: useSub ? ($("folder").value || probe.folder) : null,
    isPlaylist: useSub,
  });
  $("target").textContent = t;
}

// -------------------------------------------------------------------- analizar

async function doProbe() {
  const url = $("url").value.trim();
  if (!url) { setMsg($("probe-msg"), "Pega una URL primero.", "err"); return; }

  $("probe").disabled = true;
  setMsg($("probe-msg"), "Analizando…");
  $("info").classList.add("hidden");

  try {
    probe = await invoke("probe_url", { url });
  } catch (e) {
    setMsg($("probe-msg"), String(e), "err");
    $("probe").disabled = false;
    return;
  }
  $("probe").disabled = false;
  setMsg($("probe-msg"), "");

  const isPl = probe.kind === "playlist";
  $("kind").textContent = isPl ? `Playlist · ${probe.count} videos` : "Video";
  $("title").textContent = probe.title;
  $("meta").textContent = [
    probe.uploader,
    !isPl && probe.duration ? human(probe.duration) : null,
  ].filter(Boolean).join(" · ");
  if (probe.thumbnail) { $("thumb").src = probe.thumbnail; $("thumb").classList.remove("hidden"); }
  else $("thumb").classList.add("hidden");

  $("folder-block").classList.toggle("hidden", !isPl);
  $("conc-block").classList.toggle("hidden", !isPl);
  if (isPl) $("folder").value = probe.folder || probe.title;

  // Lista previa de la playlist
  const list = $("list");
  list.innerHTML = "";
  items.clear();
  if (isPl) {
    for (const e of probe.entries) {
      const li = document.createElement("li");
      li.innerHTML = `<span class="idx">${e.index}</span><span class="nm"></span><span class="st">${e.duration ? human(e.duration) : ""}</span>`;
      li.querySelector(".nm").textContent = e.title;
      list.appendChild(li);
      items.set(e.index, li);
    }
  }

  $("overall").classList.add("hidden");
  $("open").classList.add("hidden");
  setMsg($("status"), "");
  okCount = failCount = 0;
  $("info").classList.remove("hidden");
  await refreshDest();
}

// -------------------------------------------------------------------- descarga

function ensureItem(index, title) {
  let li = items.get(index);
  if (!li) {
    li = document.createElement("li");
    li.innerHTML = `<span class="idx">${index || "•"}</span><span class="nm"></span><span class="st"></span>`;
    $("list").appendChild(li);
    items.set(index, li);
  }
  if (title) li.querySelector(".nm").textContent = title;
  return li;
}

function updateOverall() {
  if (!probe || probe.kind !== "playlist") return;
  const total = probe.count || 1;
  const pct = Math.min(100, ((okCount + failCount) / total) * 100);
  $("overall-fill").style.width = pct + "%";
  $("overall-txt").textContent =
    `${okCount + failCount} de ${total}` + (failCount ? ` · ${failCount} con error` : "");
}

listen("download-progress", ({ payload: p }) => {
  if (p.phase === "error") {
    failCount++;
    setMsg($("status"), p.message, "err");
    updateOverall();
    return;
  }
  const li = ensureItem(p.index, p.title);
  li.classList.add("active");
  const st = li.querySelector(".st");
  if (p.phase === "progress") {
    st.textContent = `${p.percent.toFixed(0)}%` + (p.speed ? ` · ${p.speed}` : "") + (p.eta ? ` · ${p.eta}` : "");
  } else if (p.phase === "post") {
    st.textContent = p.message;
  } else if (p.phase === "done" || p.phase === "skipped") {
    li.classList.remove("active");
    li.classList.add("done");
    st.textContent = p.phase === "skipped" ? "ya estaba" : "listo";
    okCount++;
    updateOverall();
    li.scrollIntoView({ block: "nearest" });
  } else if (p.phase === "item") {
    st.textContent = "…";
  }
});

async function doDownload() {
  if (!probe || downloading) return;
  const isPl = probe.kind === "playlist";
  const useSub = isPl && $("use-folder").checked;

  downloading = true;
  $("go").classList.add("hidden");
  $("stop").classList.remove("hidden");
  $("open").classList.add("hidden");
  $("probe").disabled = true;
  okCount = failCount = 0;
  for (const li of items.values()) li.className = "";
  setMsg($("status"), "Descargando…");
  if (isPl) { $("overall").classList.remove("hidden"); updateOverall(); }

  const opts = {
    url: $("url").value.trim(),
    format: fmt(),
    quality: $("quality").value,
    dest: $("dest").value,
    folder: useSub ? ($("folder").value || probe.folder) : null,
    isPlaylist: isPl,
    concurrency: isPl ? parseInt($("conc").value, 10) : 1,
  };

  let finalPath = null;
  try {
    finalPath = await invoke("start_download", { opts });
  } catch (e) {
    setMsg($("status"), String(e), "err");
  }

  downloading = false;
  $("go").classList.remove("hidden");
  $("stop").classList.add("hidden");
  $("probe").disabled = false;

  if (finalPath === "cancelado") {
    setMsg($("status"), "Cancelado.");
  } else if (finalPath) {
    const resumen = failCount
      ? `Terminado: ${okCount} bien, ${failCount} con error.`
      : `Terminado: ${okCount || 1} ${okCount === 1 ? "archivo" : "archivos"}.`;
    setMsg($("status"), `${resumen} En ${finalPath}`, failCount ? "" : "ok");
    $("open").classList.remove("hidden");
    $("open").onclick = () => invoke("open_folder", { path: finalPath });
  }
}

// ----------------------------------------------------------------------- wiring

$("probe").onclick = doProbe;
$("url").onkeydown = (e) => { if (e.key === "Enter") doProbe(); };
$("go").onclick = doDownload;
$("stop").onclick = async () => {
  await invoke("cancel_download");
  setMsg($("status"), "Cancelando…");
};
$("pick").onclick = async () => {
  if (!openDialog) { $("dest").focus(); return; }
  const dir = await openDialog({ directory: true, multiple: false, defaultPath: $("dest").value });
  if (dir) { destTouched = true; $("dest").value = dir; await refreshTarget(); }
};
$("dest").oninput = () => { destTouched = true; refreshTarget(); };
$("folder").oninput = refreshTarget;
$("use-folder").onchange = () => {
  $("folder").disabled = !$("use-folder").checked;
  refreshTarget();
};
for (const r of document.querySelectorAll('input[name="fmt"]')) {
  r.onchange = async () => { fillQuality(); await refreshDest(); };
}

fillQuality();
refreshTools();
