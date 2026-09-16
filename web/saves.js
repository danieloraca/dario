"use strict";

// The server owns the save. Local storage remembers the player and queues unsent changes.
window.DarioSaves = (() => {
  const dialog = document.getElementById("profile-dialog");
  const form = document.getElementById("profile-form");
  const nameInput = document.getElementById("player-name");
  const error = document.getElementById("profile-error");
  const play = document.getElementById("profile-play");
  const guest = document.getElementById("profile-guest");
  const toolbar = document.getElementById("player-toolbar");
  const status = document.getElementById("save-status");
  const changePlayer = document.getElementById("change-player");
  let profile = null, pending = null, writing = null, retry, begin;
  let bytes = new Uint8Array();
  let started = false, backedUp = true;
  let phase = "title";
  const bufferId = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  const pendingKey = name => `dario.pending.${name}.${bufferId}`;
  function active() { return ["playing", "dying", "stage-clear"].includes(phase); }
  function showStatus() {
    toolbar.hidden = !started;
    changePlayer.disabled = active();
    changePlayer.title = active() ? "Pause the game to change player" : "Change player";
  }

  function readLocal(key) {
    try { return localStorage.getItem(key); } catch { return null; }
  }
  function writeLocal(key, value) {
    try {
      if (value === null) localStorage.removeItem(key); else localStorage.setItem(key, value);
      return true;
    } catch { return false; }
  }
  async function request(name, body) {
    const abort = new AbortController();
    const timeout = setTimeout(() => abort.abort(), 5000);
    try {
      const response = await fetch(`/api/progress/${name}`, {
        method: body === undefined ? "GET" : "PUT",
        headers: body === undefined ? {} : { "Content-Type": "application/json" },
        body, signal: abort.signal, cache: "no-store",
      });
      if (!response.ok) throw new Error(response.status === 409
        ? "This save could not be read. It has been preserved. Choose another player or restore a backup."
        : "Could not reach your save. Try again, or play as a guest.");
      return await response.text();
    } finally { clearTimeout(timeout); }
  }
  function flush() {
    if (writing) return writing;
    clearTimeout(retry);
    writing = (async () => {
      while (pending && profile) {
        const snapshot = pending;
        status.textContent = `${profile} · Saving...`;
        try {
          await request(profile, snapshot);
          if (pending === snapshot) {
            pending = null;
            writeLocal(pendingKey(profile), null);
          }
        } catch {
          status.textContent = backedUp ? `${profile} · Save pending, retrying` : "Save pending — keep this tab open";
          retry = setTimeout(flush, 5000);
          showStatus();
          return false;
        }
      }
      status.textContent = profile ? `${profile} · Saved` : "Guest · Progress not saved";
      showStatus();
      return true;
    })().finally(() => { writing = null; });
    return writing;
  }
  async function choose(name) {
    play.disabled = guest.disabled = true;
    error.textContent = "Loading your progress...";
    try {
      let queued = [];
      try { queued = Object.keys(localStorage).filter(key => key.startsWith(`dario.pending.${name}.`)); } catch { /* Server saving still works. */ }
      for (const key of queued) {
        const unsent = readLocal(key);
        if (unsent) {
          await request(name, unsent);
          if (readLocal(key) === unsent) writeLocal(key, null);
        }
      }
      const save = await request(name);
      writeLocal("dario.player", name);
      if (started) {
        const destination = new URL(location.href);
        destination.searchParams.set("player", name);
        location.href = destination.href;
        return;
      }
      profile = name;
      bytes = new TextEncoder().encode(save);
      dialog.close();
      start();
    } catch (cause) {
      error.textContent = cause.message;
      if (!dialog.open) dialog.showModal();
    } finally { play.disabled = guest.disabled = false; }
  }
  function start() {
    started = true;
    showStatus();
    status.textContent = profile ? `${profile} · Saved` : "Guest · Progress not saved";
    begin();
  }
  form.addEventListener("submit", event => {
    event.preventDefault();
    const name = nameInput.value.trim().toLowerCase();
    if (/^[a-z0-9_-]{1,24}$/.test(name)) void choose(name);
    else error.textContent = "Use 1–24 letters, numbers, dashes or underscores.";
  });
  dialog.addEventListener("cancel", event => { if (!started) event.preventDefault(); });
  guest.addEventListener("click", () => {
    dialog.close();
    if (started) document.getElementById("glcanvas").focus(); else start();
  });
  changePlayer.addEventListener("click", async () => {
    if (active()) return;
    const saved = await flush();
    if (active() || (!saved && !backedUp)) return;
    error.textContent = "Use the same player name on another browser to continue.";
    guest.textContent = "Back to game";
    dialog.showModal();
    nameInput.focus();
  });
  window.addEventListener("online", () => void flush());
  window.addEventListener("pagehide", () => {
    if (pending && profile) fetch(`/api/progress/${profile}`, {
      method: "PUT", headers: { "Content-Type": "application/json" }, body: pending, keepalive: true,
    }).catch(() => {});
  });
  return {
    prepare(callback) {
      begin = callback;
      const remembered = new URL(location.href).searchParams.get("player") || readLocal("dario.player");
      if (remembered && /^[a-z0-9_-]{1,24}$/.test(remembered)) {
        nameInput.value = remembered;
        void choose(remembered);
      } else { dialog.showModal(); nameInput.focus(); }
    },
    read(pointer, capacity) {
      if (bytes.length > capacity) return capacity + 1;
      new Uint8Array(wasm_memory.buffer, pointer, bytes.length).set(bytes);
      return bytes.length;
    },
    write(pointer, length) {
      if (!profile) return;
      pending = new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, pointer, length));
      backedUp = writeLocal(pendingKey(profile), pending);
      showStatus();
      void flush();
    },
    phase(value) { phase = value; showStatus(); },
  };
})();
