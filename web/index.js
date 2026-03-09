import init, { Emulator } from "./pkg/emulator.js";

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const tilesetCanvas = document.getElementById("tileset-canvas");
const tilesetCtx = tilesetCanvas.getContext("2d");
const memmapCanvas = document.getElementById("memmap-canvas");
const memmapCtx = memmapCanvas.getContext("2d");
const ilogPre = document.getElementById("ilog-pre");

const dropZone = document.getElementById("drop-zone");
const emuArea = document.getElementById("emu-area");
const romInput = document.getElementById("rom-input");
const status = document.getElementById("status");

const SCREEN_W = 160;
const SCREEN_H = 144;
const TILESET_W = 128;
const TILESET_H = 192;
const MEMMAP_W = 256;
const MEMMAP_H = 256;

let emulator = null;
let animFrame = null;

// ---------------------------------------------------------------------------
// Web Audio — ScriptProcessorNode + ring buffer.
//
// Using scheduled AudioBufferSourceNode caused a 10-15s delay because the
// AudioContext was suspended during async WASM init, so currentTime stayed
// near 0 while hundreds of rAF frames queued audio far into the future.
//
// The ring buffer approach has a hard capacity cap: if the emulator runs
// ahead, excess samples are dropped, so latency is always bounded to
// at most RING_FRAMES / 44100 ≈ 93ms.
// ---------------------------------------------------------------------------

const SAMPLE_RATE = 44100;
const SCRIPT_BUF = 2048;          // frames per callback (~46ms)
const RING_FRAMES = 4096;          // max ring depth (~93ms)
const RING_DROP_AT = RING_FRAMES * 0.8;

let audioCtx = null;
let scriptNode = null;
const ringL = new Float32Array(RING_FRAMES);
const ringR = new Float32Array(RING_FRAMES);
let writeHead = 0;
let readHead = 0;

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
            } else {
                L[i] = R[i] = 0; // underrun → silence
            }
        }
    };
    scriptNode.connect(audioCtx.destination);
}

function pushAudio(samples) {
    if (!audioCtx) return;
    const n = samples.length >> 1; // stereo interleaved → frames
    for (let i = 0; i < n; i++) {
        if (ringAvailable() >= RING_DROP_AT) break; // ring full — drop excess
        ringL[writeHead] = samples[i * 2];
        ringR[writeHead] = samples[i * 2 + 1];
        writeHead = (writeHead + 1) % RING_FRAMES;
    }
}

// ---------------------------------------------------------------------------
// Per-section debug toggle buttons
// ---------------------------------------------------------------------------

const visible = { "tileset-section": false, "memmap-section": false, "ilog-section": false };

document.querySelectorAll(".dbg-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
        const target = btn.dataset.target;
        visible[target] = !visible[target];
        document.getElementById(target).style.display = visible[target] ? "block" : "none";
        btn.classList.toggle("active", visible[target]);
    });
});

// ---------------------------------------------------------------------------
// ROM loading
// ---------------------------------------------------------------------------

async function startEmulator(romBytes) {
    await init();

    if (animFrame !== null) {
        cancelAnimationFrame(animFrame);
        animFrame = null;
    }

    // initAudio() must be called *after* a user gesture; the file picker or
    // drag-and-drop satisfies that requirement in all major browsers.
    initAudio();
    if (audioCtx.state === "suspended") await audioCtx.resume();

    emulator = new Emulator(romBytes);
    dropZone.style.display = "none";
    emuArea.style.display = "flex";
    status.textContent = "Running.";

    lastFrameTime = performance.now() - FRAME_MS; // trigger first tick immediately
    loop();
}

const TARGET_FPS = 59.7;
const FRAME_MS = 1000 / TARGET_FPS; // ~16.75ms
let lastFrameTime = 0;

function loop() {
    const now = performance.now();
    const elapsed = now - lastFrameTime;

    if (elapsed >= FRAME_MS) {
        // Preserve fractional remainder so rate stays exactly at TARGET_FPS
        // over many frames. Cap to FRAME_MS to avoid catch-up bursts.
        lastFrameTime = now - Math.min(elapsed % FRAME_MS, FRAME_MS);

        emulator.tick();

        // Main screen
        const pixels = new Uint8ClampedArray(emulator.get_framebuffer());
        ctx.putImageData(new ImageData(pixels, SCREEN_W, SCREEN_H), 0, 0);

        // Audio
        pushAudio(emulator.get_audio_samples());

        // Debug views — only computed when visible
        if (visible["tileset-section"]) {
            const tileset = new Uint8ClampedArray(emulator.get_tileset());
            tilesetCtx.putImageData(new ImageData(tileset, TILESET_W, TILESET_H), 0, 0);
        }
        if (visible["memmap-section"]) {
            const memmap = new Uint8ClampedArray(emulator.get_memory_map());
            memmapCtx.putImageData(new ImageData(memmap, MEMMAP_W, MEMMAP_H), 0, 0);
        }
        if (visible["ilog-section"]) {
            ilogPre.textContent = emulator.get_instruction_log();
        }
    }

    animFrame = requestAnimationFrame(loop);
}

// ---------------------------------------------------------------------------
// Keyboard input — also resume AudioContext on keypress as a fallback
// ---------------------------------------------------------------------------

const PREVENT_SCROLL = new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Enter"]);

window.addEventListener("keydown", (e) => {
    if (!emulator) return;
    if (PREVENT_SCROLL.has(e.key)) e.preventDefault();
    if (audioCtx && audioCtx.state === "suspended") audioCtx.resume();
    emulator.key_down(e.code);
});

window.addEventListener("keyup", (e) => {
    if (!emulator) return;
    emulator.key_up(e.code);
});

// ---------------------------------------------------------------------------
// Drag-and-drop + file picker
// ---------------------------------------------------------------------------

dropZone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropZone.classList.add("drag-over");
});
dropZone.addEventListener("dragleave", () => dropZone.classList.remove("drag-over"));
dropZone.addEventListener("drop", (e) => {
    e.preventDefault();
    dropZone.classList.remove("drag-over");
    const file = e.dataTransfer.files[0];
    if (file) loadFile(file);
});

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
