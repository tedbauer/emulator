import init, { Emulator } from "./pkg/emulator.js";

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const tilesetCanvas = document.getElementById("tileset-canvas");
const tilesetCtx = tilesetCanvas.getContext("2d");
const memmapCanvas = document.getElementById("memmap-canvas");
const memmapCtx = memmapCanvas.getContext("2d");

const dropZone = document.getElementById("drop-zone");
const emuArea = document.getElementById("emu-area");
const romInput = document.getElementById("rom-input");
const status = document.getElementById("status");
const debugPanel = document.getElementById("debug-panel");
const debugToggle = document.getElementById("debug-toggle");

const SCREEN_W = 160;
const SCREEN_H = 144;
const TILESET_W = 128;
const TILESET_H = 192;
const MEMMAP_W = 256;
const MEMMAP_H = 256;

let emulator = null;
let animFrame = null;
let debugVisible = false;

// ---------------------------------------------------------------------------
// Debug panel toggle
// ---------------------------------------------------------------------------

debugToggle.addEventListener("click", () => {
    debugVisible = !debugVisible;
    debugPanel.style.display = debugVisible ? "flex" : "none";
    debugToggle.textContent = debugVisible ? "Hide debug" : "Show debug";
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

    emulator = new Emulator(romBytes);

    dropZone.style.display = "none";
    emuArea.style.display = "flex";
    status.textContent = "Running.";

    loop();
}

function loop() {
    emulator.tick();

    // Main screen
    const pixels = new Uint8ClampedArray(emulator.get_framebuffer());
    ctx.putImageData(new ImageData(pixels, SCREEN_W, SCREEN_H), 0, 0);

    // Debug views — only generated when panel is visible (avoid pointless work)
    if (debugVisible) {
        const tileset = new Uint8ClampedArray(emulator.get_tileset());
        tilesetCtx.putImageData(new ImageData(tileset, TILESET_W, TILESET_H), 0, 0);

        const memmap = new Uint8ClampedArray(emulator.get_memory_map());
        memmapCtx.putImageData(new ImageData(memmap, MEMMAP_W, MEMMAP_H), 0, 0);
    }

    animFrame = requestAnimationFrame(loop);
}

// ---------------------------------------------------------------------------
// Keyboard input
// ---------------------------------------------------------------------------

const PREVENT_SCROLL = new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Enter"]);

window.addEventListener("keydown", (e) => {
    if (!emulator) return;
    if (PREVENT_SCROLL.has(e.key)) e.preventDefault();
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
