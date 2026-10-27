import init, { LearnTrace } from "./pkg/iqrcode_learn_wasm.js";
import { en } from "./i18n/en.js";
import { fr } from "./i18n/fr.js";

const dictionaries = { fr, en };
const storageKey = "iqrcode-learn-language";

const ui = {
  data: document.getElementById("data"),
  level: document.getElementById("level"),
  version: document.getElementById("version"),
  lang: document.getElementById("lang"),
  regenerate: document.getElementById("regenerate"),
  canvas: document.getElementById("canvas"),
  meta: document.getElementById("meta"),
  title: document.getElementById("title"),
  description: document.getElementById("description"),
  mask: document.getElementById("mask"),
  lessonFormula: document.getElementById("lesson-formula"),
  prev: document.getElementById("prev"),
  play: document.getElementById("play"),
  next: document.getElementById("next"),
  speed: document.getElementById("speed"),
  progress: document.getElementById("progress"),
  pipeline: document.getElementById("pipeline"),
  error: document.getElementById("error"),
};

let trace = null;
let frameIndex = 0;
let playing = false;
let timer = null;
let language =
  localStorage.getItem(storageKey) ??
  (navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en");

function t() {
  return dictionaries[language] ?? dictionaries.en;
}

async function boot() {
  await init();
  applyTranslations();
  bindControls();
  regenerate();
}

function bindControls() {
  ui.regenerate.addEventListener("click", regenerate);
  ui.data.addEventListener("keydown", (event) => {
    if (event.key === "Enter") regenerate();
  });
  ui.level.addEventListener("change", regenerate);
  ui.version.addEventListener("change", regenerate);
  ui.lang.addEventListener("change", changeLanguage);
  ui.prev.addEventListener("click", () => step(-1));
  ui.next.addEventListener("click", () => step(1));
  ui.play.addEventListener("click", () => (playing ? pause() : play()));
  ui.progress.addEventListener("input", () => {
    pause();
    frameIndex = Number(ui.progress.value);
    render();
  });
  window.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft") step(-1);
    if (event.key === "ArrowRight") step(1);
    if (event.key === " ") {
      event.preventDefault();
      playing ? pause() : play();
    }
  });
}

function applyTranslations() {
  const dictionary = t();

  document.documentElement.lang = language;
  ui.lang.value = language;
  ui.version.placeholder = dictionary.ui.auto;
  ui.prev.title = dictionary.ui.previous;
  ui.next.title = dictionary.ui.next;
  ui.play.textContent = playing ? dictionary.ui.pause : dictionary.ui.play;

  document.querySelectorAll("[data-i18n]").forEach((element) => {
    const key = element.dataset.i18n;
    const value = dictionary.ui[key];

    if (typeof value === "string") {
      element.textContent = value;
    }
  });
}

function changeLanguage() {
  language = ui.lang.value;
  localStorage.setItem(storageKey, language);
  applyTranslations();
  renderPipeline();

  if (trace) {
    render();
  }
}

function regenerate() {
  pause();
  clearError();

  if (trace) {
    trace.free();
    trace = null;
  }

  const data = ui.data.value || "HELLO WORLD";
  const level = ui.level.value;
  const versionText = ui.version.value.trim();
  const version = versionText ? Number(versionText) : undefined;

  try {
    trace = new LearnTrace(data, level, version, 0, 300, true);
    frameIndex = 0;
    renderPipeline();
    render();
  } catch (error) {
    showError(error);
  }
}

function renderPipeline() {
  ui.pipeline.innerHTML = "";

  for (const lesson of t().pipeline) {
    const item = document.createElement("li");
    const title = document.createElement("strong");
    title.textContent = lesson.title;
    item.appendChild(title);

    const explanation = document.createElement("p");
    explanation.textContent = lesson.explanation;
    item.appendChild(explanation);

    if (lesson.formula) {
      const pre = document.createElement("pre");
      pre.textContent = lesson.formula;
      item.appendChild(pre);
    }

    ui.pipeline.appendChild(item);
  }
}

function render() {
  if (!trace) return;

  const total = trace.frame_count();
  if (total === 0) return;

  frameIndex = Math.max(0, Math.min(frameIndex, total - 1));

  const frame = trace.frame(frameIndex);
  const size = frame.size();
  const quiet = 4;
  const scale = Math.max(2, Math.min(12, Math.floor(560 / (size + 2 * quiet))));
  const dimension = (size + 2 * quiet) * scale;
  const context = ui.canvas.getContext("2d");

  ui.canvas.width = dimension;
  ui.canvas.height = dimension;

  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, dimension, dimension);

  const matrix = frame.matrix();
  const highlight = frame.highlight();
  const highlighted = new Set();

  for (let index = 0; index < highlight.length; index += 2) {
    highlighted.add(`${highlight[index]},${highlight[index + 1]}`);
  }

  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const dark = matrix[y * size + x] === 1;
      const isHighlighted = highlighted.has(`${x},${y}`);

      if (!dark && !isHighlighted) continue;

      context.fillStyle = isHighlighted
        ? dark
          ? "#d21f3c"
          : "rgba(210, 31, 60, 0.35)"
        : "#000000";

      context.fillRect(
        (x + quiet) * scale,
        (y + quiet) * scale,
        scale,
        scale,
      );
    }
  }

  const dictionary = t();
  const phase = frame.phase();
  const phaseText = dictionary.phases[phase] ?? dictionary.phases.unknown;
  const title = resolve(phaseText.title, frame, trace);
  const description = resolve(phaseText.description, frame, trace);
  const formula = resolve(phaseText.formula, frame, trace);

  ui.meta.textContent = dictionary.ui.meta(trace);
  ui.title.textContent = dictionary.ui.frameTitle(title, frameIndex, total);
  ui.description.textContent = description;
  ui.lessonFormula.textContent = formula;
  ui.mask.textContent = dictionary.ui.mask(frame);
  ui.progress.max = Math.max(1, total - 1);
  ui.progress.value = frameIndex;

  frame.free();
}

function resolve(value, ...args) {
  if (typeof value === "function") {
    return value(...args);
  }

  return value ?? "";
}

function play() {
  if (!trace || playing) return;

  if (frameIndex >= trace.frame_count() - 1) {
    frameIndex = 0;
    render();
  }

  playing = true;
  ui.play.textContent = t().ui.pause;
  schedule();
}

function pause() {
  playing = false;
  clearTimeout(timer);
  ui.play.textContent = t().ui.play;
}

function schedule() {
  if (!playing || !trace) return;

  timer = setTimeout(() => {
    if (!trace) return;

    if (frameIndex < trace.frame_count() - 1) {
      frameIndex += 1;
      render();
      schedule();
    } else {
      pause();
    }
  }, Number(ui.speed.value));
}

function step(delta) {
  if (!trace) return;

  pause();
  frameIndex = Math.max(0, Math.min(frameIndex + delta, trace.frame_count() - 1));
  render();
}

function showError(error) {
  ui.error.textContent = t().ui.errorPrefix + String(error);
}

function clearError() {
  ui.error.textContent = "";
}

boot().catch(showError);
