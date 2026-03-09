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

const codeEditor = document.getElementById("code-editor");
const compileBtn = document.getElementById("compile-btn");
const compileError = document.getElementById("compile-error");

// ── Dimensions ────────────────────────────────────────────────────────────────
const SCREEN_W = 160;
const SCREEN_H = 144;
const TILESET_W = 128;
const TILESET_H = 192;
const MEMMAP_W = 256;
const MEMMAP_H = 256;

// ── Default Shrimp source (pong) ──────────────────────────────────────────────
codeEditor.value = `from core import pressed, set_sprite, Button

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

// Tab key inserts spaces in the editor
codeEditor.addEventListener("keydown", (e) => {
    if (e.key === "Tab") {
        e.preventDefault();
        const s = codeEditor.selectionStart;
        codeEditor.value = codeEditor.value.slice(0, s) + "    " + codeEditor.value.slice(codeEditor.selectionEnd);
        codeEditor.selectionStart = codeEditor.selectionEnd = s + 4;
    }
});

// ── State ─────────────────────────────────────────────────────────────────────
let emulator = null;
let animFrame = null;

// ── Web Audio ─────────────────────────────────────────────────────────────────
const SAMPLE_RATE = 44100;
const SCRIPT_BUF = 2048;
const RING_FRAMES = 4096;
const RING_DROP_AT = RING_FRAMES * 0.8;

let audioCtx = null;
let scriptNode = null;
const ringL = new Float32Array(RING_FRAMES);
const ringR = new Float32Array(RING_FRAMES);
let writeHead = 0, readHead = 0;

function ringAvailable() {
    return (writeHead - readHead + RING_FRAMES) % RING_FRAMES;
}

function initAudio() {
    if (audioCtx) return;
    audioCtx = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: SAMPLE_RATE });
    scriptNode = audioCtx.createScriptProcessor(SCRIPT_BUF, 0, 2);
    scriptNode.onaudioprocess = ({ outputBuffer }) => {
        const L = outputBuffer.getChannelData(0);
        const R = outputBuffer.getChannelData(1);
        for (let i = 0; i < L.length; i++) {
            if (ringAvailable() > 0) {
                L[i] = ringL[readHead];
                R[i] = ringR[readHead];
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
const visible = {
    "tileset-section": false,
    "memmap-section": false,
    "ilog-section": false,
};

document.querySelectorAll(".dbg-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
        const target = btn.dataset.target;
        visible[target] = !visible[target];
        document.getElementById(target).style.display = visible[target] ? "block" : "none";
        btn.classList.toggle("active", visible[target]);
    });
});

// ── Emulator loop ─────────────────────────────────────────────────────────────
const TARGET_FPS = 59.7;
const FRAME_MS = 1000 / TARGET_FPS;
let lastFrameTime = 0;

function startEmulatorLoop() {
    if (animFrame !== null) { cancelAnimationFrame(animFrame); animFrame = null; }
    lastFrameTime = performance.now() - FRAME_MS;
    loop();
}

function loop() {
    const now = performance.now();
    const elapsed = now - lastFrameTime;

    if (elapsed >= FRAME_MS) {
        lastFrameTime = now - Math.min(elapsed % FRAME_MS, FRAME_MS);
        emulator.tick();

        const pixels = new Uint8ClampedArray(emulator.get_framebuffer());
        ctx.putImageData(new ImageData(pixels, SCREEN_W, SCREEN_H), 0, 0);
        pushAudio(emulator.get_audio_samples());

        if (visible["tileset-section"]) {
            const t = new Uint8ClampedArray(emulator.get_tileset());
            tilesetCtx.putImageData(new ImageData(t, TILESET_W, TILESET_H), 0, 0);
        }
        if (visible["memmap-section"]) {
            const m = new Uint8ClampedArray(emulator.get_memory_map());
            memmapCtx.putImageData(new ImageData(m, MEMMAP_W, MEMMAP_H), 0, 0);
        }
        if (visible["ilog-section"]) {
            ilogPre.textContent = emulator.get_instruction_log();
        }
    }

    animFrame = requestAnimationFrame(loop);
}

async function startEmulator(romBytes) {
    if (animFrame !== null) { cancelAnimationFrame(animFrame); animFrame = null; }
    initAudio();
    if (audioCtx.state === "suspended") await audioCtx.resume();
    emulator = new Emulator(romBytes);
    lastFrameTime = performance.now() - FRAME_MS;
    loop();
}

// ── Compile & Run ─────────────────────────────────────────────────────────────
compileBtn.addEventListener("click", async () => {
    compileError.classList.remove("visible");
    compileError.textContent = "";
    status.textContent = "Compiling…";

    initAudio();
    if (audioCtx.state === "suspended") await audioCtx.resume();

    let romBytes;
    try {
        romBytes = compile_to_rom(codeEditor.value);
    } catch (err) {
        compileError.textContent = String(err);
        compileError.classList.add("visible");
        status.textContent = "Compile error.";
        return;
    }

    try {
        await startEmulator(romBytes);
        status.textContent = "Running.";
    } catch (err) {
        status.textContent = `Emulator error: ${err}`;
        console.error(err);
    }
});

// ── Keyboard ──────────────────────────────────────────────────────────────────
const PREVENT_SCROLL = new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Enter"]);

window.addEventListener("keydown", (e) => {
    // Don't intercept keys while typing in the code editor
    if (document.activeElement === codeEditor) return;
    if (!emulator) return;
    if (PREVENT_SCROLL.has(e.key)) e.preventDefault();
    if (audioCtx && audioCtx.state === "suspended") audioCtx.resume();
    emulator.key_down(e.code);
});

window.addEventListener("keyup", (e) => {
    if (document.activeElement === codeEditor) return;
    if (!emulator) return;
    emulator.key_up(e.code);
});

// ── File drop / picker ────────────────────────────────────────────────────────

romInput.addEventListener("change", () => {
    const file = romInput.files[0];
    if (file) loadFile(file);
});

function loadFile(file) {
    status.textContent = `Loading ${file.name}…`;
    const reader = new FileReader();
    reader.onload = (e) => {
        const bytes = new Uint8Array(e.target.result);
        startEmulator(bytes).catch((err) => {
            status.textContent = `Error: ${err}`;
            console.error(err);
        });
    };
    reader.readAsArrayBuffer(file);
}

// ── WASM init ─────────────────────────────────────────────────────────────────
(async () => {
    status.textContent = "Loading…";
    try {
        await Promise.all([initEmu(), initComp()]);
        compileBtn.disabled = false;
        status.textContent = "Load a ROM to start.";
    } catch (err) {
        status.textContent = `Failed to load: ${err}`;
        console.error(err);
    }
})();
