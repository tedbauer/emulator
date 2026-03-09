import initEmu, { Emulator } from "./pkg/emulator.js";
import initComp, { compile_to_rom } from "./compiler_pkg/compiler.js";

// ── DOM refs ──────────────────────────────────────────────────────────────────
const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const tilesetCanvas = document.getElementById("tileset-canvas");
const tilesetCtx = tilesetCanvas.getContext("2d");
const memmapCanvas = document.getElementById("memmap-canvas");
const memmapCtx = memmapCanvas.getContext("2d");
const ilogPre = document.getElementById("ilog-pre");

const romInput = document.getElementById("rom-input");
const status = document.getElementById("status");
const placeholder = document.getElementById("screen-placeholder");

const codeEditor = document.getElementById("code-editor");
const lineNums = document.getElementById("line-numbers");
const runBtn = document.getElementById("run-btn");
const newFileBtn = document.getElementById("new-file-btn");
const demoBtn = document.getElementById("demo-btn");
const demoPicker = document.getElementById("demo-picker");
const compileError  = document.getElementById("compile-error");
const tabBar        = document.getElementById("tab-bar");
const termOutput    = document.getElementById("terminal-output");
const editorBody    = document.getElementById("editor-body");
const editorEmpty   = document.getElementById("editor-empty");

// ── Dimensions ────────────────────────────────────────────────────────────────
const SCREEN_W = 160;
const SCREEN_H = 144;
const TILESET_W = 128;
const TILESET_H = 192;
const MEMMAP_W = 256;
const MEMMAP_H = 256;

// ── Example sources ───────────────────────────────────────────────────────────
const PONG_SOURCE = `from core import pressed, set_sprite, Button

tile ball:
    ..3333..
    .333333.
    33333333
    33333333
    33333333
    33333333
    .333333.
    ..3333..

tile paddle:
    33333333
    33333333
    33333333
    33333333
    ........
    ........
    ........
    ........

let bx = 80
let by = 72
let vx: i8 = 1
let vy: i8 = 1
let px = 64

init:
    set_sprite(0, bx, by, ball)
    set_sprite(1, px, 136, paddle)
    set_sprite(2, px + 8, 136, paddle)
    set_sprite(3, px + 16, 136, paddle)

on vblank:
    bx := bx + vx
    by := by + vy

    if bx <= 1:
        vx := 1
    if bx >= 152:
        vx := -1

    if by <= 2:
        vy := 1

    if by >= 126:
        if by <= 136:
            if bx >= px:
                if bx <= px + 24:
                    vy := -1

    if by >= 144:
        bx := 80
        by := 72
        vx := 1
        vy := 1

    if pressed(Button.LEFT):
        if px >= 2:
            px := px - 2
    if pressed(Button.RIGHT):
        if px <= 136:
            px := px + 2

    set_sprite(0, bx, by, ball)
    set_sprite(1, px, 136, paddle)
    set_sprite(2, px + 8, 136, paddle)
    set_sprite(3, px + 16, 136, paddle)
`;

const PLATFORMER_SOURCE = `from core import pressed, set_sprite, set_bg_tile, Button

tile player:
    ..3333..
    .333333.
    .3.33.3.
    .333333.
    ..3333..
    .3.33.3.
    .3....3.
    ........

tile ground:
    33333333
    22222222
    22222222
    22222222
    22222222
    22222222
    22222222
    22222222

tile platform:
    33333333
    33333333
    22222222
    ........
    ........
    ........
    ........
    ........

let px: u8 = 24
let py: u8 = 100
let vy: i8 = 0
let on_ground: u8 = 0
let jump_lock: u8 = 0
let tx: u8 = 0

init:
    while tx <= 19:
        set_bg_tile(tx, 16, ground)
        tx := tx + 1
    set_bg_tile(2, 12, platform)
    set_bg_tile(3, 12, platform)
    set_bg_tile(4, 12, platform)
    set_bg_tile(9, 10, platform)
    set_bg_tile(10, 10, platform)
    set_bg_tile(11, 10, platform)
    set_bg_tile(14, 8, platform)
    set_bg_tile(15, 8, platform)
    set_bg_tile(16, 8, platform)
    set_sprite(0, px, py, player)

on vblank:
    if on_ground == 0:
        vy := vy + 1
    if jump_lock >= 1:
        jump_lock := jump_lock - 1

    if on_ground == 1:
        if pressed(Button.A):
            vy := -8
            on_ground := 0
            jump_lock := 12

    if pressed(Button.LEFT):
        if px >= 2:
            px := px - 2
    if pressed(Button.RIGHT):
        if px <= 150:
            px := px + 2

    py := py + vy
    on_ground := 0

    if jump_lock == 0:
        if px >= 12:
            if px <= 42:
                if py >= 88:
                    if py <= 102:
                        py := 88
                        vy := 0
                        on_ground := 1

    if jump_lock == 0:
        if on_ground == 0:
            if px >= 68:
                if px <= 96:
                    if py >= 72:
                        if py <= 86:
                            py := 72
                            vy := 0
                            on_ground := 1

    if jump_lock == 0:
        if on_ground == 0:
            if px >= 108:
                if px <= 136:
                    if py >= 56:
                        if py <= 70:
                            py := 56
                            vy := 0
                            on_ground := 1

    if on_ground == 0:
        if py >= 120:
            py := 120
            vy := 0
            on_ground := 1

    set_sprite(0, px, py, player)
`;

// ── File system ───────────────────────────────────────────────────────────────
let files    = [];   // [{ id, name, content }]
let activeId = null;
let nextId   = 0;
let untitledCounter = 1;
let compilerReady   = false;
let runningId       = null;  // id of the file currently in the emulator

function createFile(name, content = "") {
    const id = nextId++;
    files.push({ id, name, content });
    return id;
}

function activeFile() {
    return files.find(f => f.id === activeId) ?? null;
}

function saveActiveContent() {
    const f = activeFile();
    if (f) f.content = codeEditor.value;
}

function updateEditorState() {
    const hasFiles = files.length > 0;
    editorBody.classList.toggle("hidden", !hasFiles);
    editorEmpty.classList.toggle("hidden", hasFiles);
    runBtn.disabled = !(hasFiles && compilerReady);
}

function switchTo(id) {
    saveActiveContent();
    activeId = id;
    const f = activeFile();
    codeEditor.value = f ? f.content : "";
    updateLineNumbers();
    renderTabs();
    updateEditorState();
    document.title = f ? `${f.name} — Shrimp` : "Shrimp Editor";
}

function closeFile(id) {
    saveActiveContent();
    const idx = files.findIndex(f => f.id === id);
    if (idx === -1) return;
    files.splice(idx, 1);
    if (files.length === 0) {
        activeId = null;
        codeEditor.value = "";
        renderTabs();
        updateEditorState();
        document.title = "Shrimp Editor";
    } else {
        switchTo(files[Math.min(idx, files.length - 1)].id);
    }
}

// ── Tab rendering ─────────────────────────────────────────────────────────────
function renderTabs() {
    tabBar.innerHTML = "";
    for (const f of files) {
        const tab = document.createElement("div");
        tab.className = "tab" +
            (f.id === activeId  ? " active"  : "") +
            (f.id === runningId ? " running" : "");

        const nameSpan = document.createElement("span");
        nameSpan.className = "tab-name";
        nameSpan.textContent = f.name;

        const closeBtn = document.createElement("button");
        closeBtn.className = "tab-close";
        closeBtn.textContent = "×";
        closeBtn.title = "Close";
        closeBtn.addEventListener("click", e => { e.stopPropagation(); closeFile(f.id); });

        tab.appendChild(nameSpan);
        tab.appendChild(closeBtn);
        tab.addEventListener("click", () => switchTo(f.id));
        tabBar.appendChild(tab);
    }
}

// ── Sidebar actions ───────────────────────────────────────────────────────────

// + New — reuse an existing empty untitled tab if one exists
newFileBtn.addEventListener("click", () => {
    const existing = files.find(f => f.name.startsWith("untitled-") && f.content.trim() === "");
    if (existing) { switchTo(existing.id); return; }
    switchTo(createFile(`untitled-${untitledCounter++}.s`, ""));
});

// Demo button toggles picker
demoBtn.addEventListener("click", e => {
    e.stopPropagation();
    demoPicker.classList.toggle("hidden");
});

// Picker items load the demo
const DEMO_SOURCES = { "pong.s": PONG_SOURCE, "platformer.s": PLATFORMER_SOURCE };
document.querySelectorAll(".picker-item[data-name]").forEach(item => {
    item.addEventListener("click", () => {
        demoPicker.classList.add("hidden");
        const name = item.dataset.name;
        const existing = files.find(f => f.name === name);
        if (existing) { switchTo(existing.id); return; }
        switchTo(createFile(name, DEMO_SOURCES[name] ?? ""));
    });
});

// Close picker when clicking elsewhere
document.addEventListener("click", () => demoPicker.classList.add("hidden"));

// ── Line numbers ──────────────────────────────────────────────────────────────
function updateLineNumbers() {
    const count = (codeEditor.value.match(/\n/g) || []).length + 1;
    let out = "";
    for (let i = 1; i <= count; i++) out += i + "\n";
    lineNums.textContent = out;
    lineNums.scrollTop = codeEditor.scrollTop;
}

codeEditor.addEventListener("input", updateLineNumbers);
codeEditor.addEventListener("scroll", () => { lineNums.scrollTop = codeEditor.scrollTop; });

// ── Tab key in editor ─────────────────────────────────────────────────────────
codeEditor.addEventListener("keydown", e => {
    if (e.key === "Tab") {
        e.preventDefault();
        const s = codeEditor.selectionStart;
        codeEditor.value = codeEditor.value.slice(0, s) + "    " + codeEditor.value.slice(codeEditor.selectionEnd);
        codeEditor.selectionStart = codeEditor.selectionEnd = s + 4;
    }
});

// ── Terminal ──────────────────────────────────────────────────────────────────
function termClear() { termOutput.innerHTML = ""; }

function termLine(text, cls = "") {
    const span = document.createElement("span");
    span.className = "term-line" + (cls ? " " + cls : "");
    span.textContent = text;
    termOutput.appendChild(span);
    termOutput.scrollTop = termOutput.scrollHeight;
}

// ── Audio ─────────────────────────────────────────────────────────────────────
const SAMPLE_RATE = 44100;
const SCRIPT_BUF = 2048;
const RING_FRAMES = 4096;
const RING_DROP_AT = RING_FRAMES * 0.8;

let audioCtx = null;
let scriptNode = null;
const ringL = new Float32Array(RING_FRAMES);
const ringR = new Float32Array(RING_FRAMES);
let writeHead = 0, readHead = 0;

function ringAvailable() { return (writeHead - readHead + RING_FRAMES) % RING_FRAMES; }

function initAudio() {
    if (audioCtx) return;
    audioCtx = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: SAMPLE_RATE });
    scriptNode = audioCtx.createScriptProcessor(SCRIPT_BUF, 0, 2);
    scriptNode.onaudioprocess = ({ outputBuffer }) => {
        const L = outputBuffer.getChannelData(0);
        const R = outputBuffer.getChannelData(1);
        for (let i = 0; i < L.length; i++) {
            if (ringAvailable() > 0) {
                L[i] = ringL[readHead]; R[i] = ringR[readHead];
                readHead = (readHead + 1) % RING_FRAMES;
            } else { L[i] = R[i] = 0; }
        }
    };
    scriptNode.connect(audioCtx.destination);
}

function pushAudio(samples) {
    if (!audioCtx) return;
    const n = samples.length >> 1;
    for (let i = 0; i < n; i++) {
        if (ringAvailable() >= RING_DROP_AT) break;
        ringL[writeHead] = samples[i * 2];
        ringR[writeHead] = samples[i * 2 + 1];
        writeHead = (writeHead + 1) % RING_FRAMES;
    }
}

// ── Debug toggles ─────────────────────────────────────────────────────────────
const visible = { "tileset-section": false, "memmap-section": false, "ilog-section": false };

document.querySelectorAll(".dbg-btn").forEach(btn => {
    btn.addEventListener("click", () => {
        const target = btn.dataset.target;
        visible[target] = !visible[target];
        document.getElementById(target).style.display = visible[target] ? "block" : "none";
        btn.classList.toggle("active", visible[target]);
    });
});

// ── Emulator loop ─────────────────────────────────────────────────────────────
let emulator = null;
let animFrame = null;
let lastFrame = 0;
const TARGET_FPS = 59.7;
const FRAME_MS = 1000 / TARGET_FPS;

// Pre-allocated render buffers (avoids per-frame GC pressure)
let screenBuf    = null;  // Uint8ClampedArray
let screenImg    = null;  // ImageData
let tilesetBuf   = null;
let tilesetImg   = null;
let memmapBuf    = null;
let memmapImg    = null;

function loop(now) {
    const elapsed = now - lastFrame;
    if (elapsed >= FRAME_MS) {
        lastFrame = now - Math.min(elapsed % FRAME_MS, FRAME_MS);
        emulator.tick();
        screenBuf.set(emulator.get_framebuffer());
        ctx.putImageData(screenImg, 0, 0);
        pushAudio(emulator.get_audio_samples());
        if (visible["tileset-section"]) {
            tilesetBuf.set(emulator.get_tileset());
            tilesetCtx.putImageData(tilesetImg, 0, 0);
        }
        if (visible["memmap-section"]) {
            memmapBuf.set(emulator.get_memory_map());
            memmapCtx.putImageData(memmapImg, 0, 0);
        }
        if (visible["ilog-section"])
            ilogPre.textContent = emulator.get_instruction_log();
    }
    animFrame = requestAnimationFrame(loop);
}

async function startEmulator(romBytes) {
    if (animFrame !== null) { cancelAnimationFrame(animFrame); animFrame = null; }
    initAudio();
    if (audioCtx.state === "suspended") await audioCtx.resume();
    emulator = new Emulator(romBytes);
    // Allocate render buffers once per emulator session
    screenBuf  = new Uint8ClampedArray(SCREEN_W  * SCREEN_H  * 4);
    screenImg  = new ImageData(screenBuf, SCREEN_W, SCREEN_H);
    tilesetBuf = new Uint8ClampedArray(TILESET_W * TILESET_H * 4);
    tilesetImg = new ImageData(tilesetBuf, TILESET_W, TILESET_H);
    memmapBuf  = new Uint8ClampedArray(MEMMAP_W  * MEMMAP_H  * 4);
    memmapImg  = new ImageData(memmapBuf, MEMMAP_W, MEMMAP_H);
    placeholder.classList.add("hidden");
    lastFrame = performance.now() - FRAME_MS;
    animFrame = requestAnimationFrame(loop);
}

// ── Run button ────────────────────────────────────────────────────────────────
runBtn.addEventListener("click", async () => {
    saveActiveContent();
    const f = activeFile();
    if (!f) return;

    compileError.classList.remove("visible");
    compileError.textContent = "";
    termClear();

    initAudio();
    if (audioCtx.state === "suspended") await audioCtx.resume();

    const t0 = performance.now();
    termLine(`🦐  Compiling ${f.name}…`, "term-dim");
    status.textContent = "Compiling…";

    let romBytes;
    try {
        romBytes = compile_to_rom(f.content);
    } catch (err) {
        const msg = String(err);
        compileError.textContent = msg;
        compileError.classList.add("visible");
        termLine(`✗  ${msg}`, "term-err");
        status.textContent = "Compile error.";
        return;
    }

    const ms = (performance.now() - t0).toFixed(0);
    termLine(`✓  Generated ${romBytes.length.toLocaleString()} bytes in ${ms}ms`, "term-ok");

    try {
        await startEmulator(romBytes);
        runningId = f.id;
        renderTabs();
        termLine(`▶  Running ${f.name} in emulator`, "term-info");
        status.textContent = "Running.";
    } catch (err) {
        termLine(`✗  Emulator error: ${err}`, "term-err");
        status.textContent = "Emulator error.";
        console.error(err);
    }
});

// ── Keyboard ──────────────────────────────────────────────────────────────────
const PREVENT_SCROLL = new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);

window.addEventListener("keydown", e => {
    if (document.activeElement === codeEditor) return;
    if (!emulator) return;
    if (PREVENT_SCROLL.has(e.key)) e.preventDefault();
    if (audioCtx && audioCtx.state === "suspended") audioCtx.resume();
    emulator.key_down(e.code);
});
window.addEventListener("keyup", e => {
    if (document.activeElement === codeEditor) return;
    if (!emulator) return;
    emulator.key_up(e.code);
});

// ── Load .gb file ─────────────────────────────────────────────────────────────
romInput.addEventListener("change", () => {
    const file = romInput.files[0];
    if (!file) return;
    status.textContent = `Loading ${file.name}…`;
    const reader = new FileReader();
    reader.onload = e => {
        const bytes = new Uint8Array(e.target.result);
        termClear();
        termLine(`📂  Loaded ${file.name} (${bytes.length.toLocaleString()} bytes)`, "term-info");
        startEmulator(bytes)
            .then(() => { status.textContent = "Running."; termLine("▶  Running in emulator", "term-ok"); })
            .catch(err => { status.textContent = `Error: ${err}`; console.error(err); });
    };
    reader.readAsArrayBuffer(file);
    // Reset so the same file can be re-loaded if needed
    romInput.value = "";
});

// ── WASM init ─────────────────────────────────────────────────────────────────
(async () => {
    status.textContent = "Loading…";
    try {
        await Promise.all([initEmu(), initComp()]);
        compilerReady = true;
        status.textContent = "Ready.";
        // Open Pong by default
        switchTo(createFile("pong.s", PONG_SOURCE));
        termLine("🦐  Shrimp compiler ready", "term-ok");
        termLine("    Press ▶ Run to compile and play", "term-dim");
    } catch (err) {
        status.textContent = `Failed to load: ${err}`;
        termLine(`✗  Load failed: ${err}`, "term-err");
        console.error(err);
    }
})();
