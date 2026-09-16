"use strict";

(() => {
  const gameCanvas = document.getElementById("glcanvas");
  const loading = document.getElementById("loading");
  const message = document.getElementById("loading-message");
  const retry = document.getElementById("retry");
  const phases = ["title", "playing", "paused", "dying", "stage-clear", "game-over", "won"];
  let ready = false;
  let pauseRequested = false;
  let stopSound;
  const playedSounds = new Set();

  function fail(text) {
    loading.hidden = false;
    message.textContent = text;
    retry.hidden = false;
    gameCanvas.dataset.state = "error";
  }
  if (typeof WebAssembly !== "object" || typeof miniquad_add_plugin !== "function") {
    fail("The game could not load. Check your connection and use a browser with WebAssembly support.");
    return;
  }
  const loadingTimeout = setTimeout(() => {
    if (!ready) fail("The game is taking too long to start. Reload the page and check that WebGL is available.");
  }, 20000);
  window.addEventListener("error", () => {
    clearTimeout(loadingTimeout);
    fail("Something stopped the game. Reload the page to try again.");
  });

  miniquad_add_plugin({
    name: "dario_web",
    version: 1,
    register_plugin(imports) {
      imports.env.dario_status = (phase, world, coins, lives, muted) => {
        ready = true;
        clearTimeout(loadingTimeout);
        loading.hidden = true;
        Object.assign(gameCanvas.dataset, { state: phases[phase], world, coins, lives, muted: Boolean(muted) });
        gameCanvas.setAttribute("aria-label", `Dario: ${phases[phase]}. World 1-${world}, ${coins} coins, ${lives} lives. Sound ${muted ? "off" : "on"}.`);
      };
      imports.env.dario_take_pause_request = () => {
        const requested = pauseRequested;
        pauseRequested = false;
        return Number(requested);
      };
      // Silence a hidden tab immediately, even when animation frames are suspended.
      const playSound = imports.env.audio_play_buffer;
      stopSound = imports.env.audio_source_stop;
      imports.env.audio_play_buffer = (sound, volume, repeat) => {
        playedSounds.add(sound);
        return playSound(sound, volume, repeat);
      };
    },
  });

  function pause() {
    pauseRequested = true;
    if (stopSound) for (const sound of playedSounds) stopSound(sound);
  }
  window.addEventListener("blur", pause);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) pause();
  });
  gameCanvas.addEventListener("pointerdown", () => gameCanvas.focus());
  gameCanvas.addEventListener("keydown", (event) => {
    if (event.code === "KeyF" && !event.repeat && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      const operation = document.fullscreenElement ? document.exitFullscreen() : gameCanvas.requestFullscreen?.();
      operation?.catch(() => { /* Embedded pages may not grant fullscreen permission. */ });
    }
  });

  // Both runtime and WASM are served locally from the locked Cargo dependencies.
  load("dario.wasm");
})();
