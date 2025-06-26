import init, { Emulator } from './pkg/emulator.js';

// --- Constants ---
const SCREEN_WIDTH = 160;
const SCREEN_HEIGHT = 144;
const FRAMEBUFFER_SIZE = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
const TILESET_WIDTH = 128;
const TILESET_HEIGHT = 192;
const TILESET_BUFFER_SIZE = TILESET_WIDTH * TILESET_HEIGHT * 4;

// --- Get DOM Elements ---
const canvas = document.getElementById('emulator-canvas');
const ctx = canvas.getContext('2d');
const tilesetCanvas = document.getElementById('tileset-canvas');
const tilesetCtx = tilesetCanvas.getContext('2d');
const speedSlider = document.getElementById('speed-slider');
const speedValue = document.getElementById('speed-value');

async function run() {
    console.log("run() function called. Awaiting init()...");

    const wasmModule = await init();
    console.log("init() successful. WASM module loaded.");

    const emu = new Emulator(new Uint8Array());
    const memory = wasmModule.memory;

    // --- SPEED CONTROL LOGIC ---
    let slowdownFactor = 40; // CHANGED: Default to a much slower speed
    let frameCounter = 0;

    // Set initial text and slider value
    speedSlider.value = slowdownFactor;
    speedValue.textContent = `${slowdownFactor}x Slower`;

    // Listen for changes on the slider
    speedSlider.addEventListener('input', (event) => {
        slowdownFactor = parseInt(event.target.value, 10);
        if (slowdownFactor === 1) {
            speedValue.textContent = 'Full Speed';
        } else {
            speedValue.textContent = `${slowdownFactor}x Slower`;
        }
    });
    // --- END OF SPEED CONTROL LOGIC ---

    // --- The Main Render Loop ---
    const renderLoop = () => {
        frameCounter++;

        // Only execute the emulator tick and draw if the counter
        // is a multiple of our slowdown factor.
        if (frameCounter % slowdownFactor === 0) {
            // Run the emulator for one full frame
            emu.tick();

            // --- Draw the Main Game Screen ---
            const framebufferPtr = emu.framebuffer_ptr();
            const pixelData = new Uint8ClampedArray(memory.buffer, framebufferPtr, FRAMEBUFFER_SIZE);
            const imageData = new ImageData(pixelData, SCREEN_WIDTH, SCREEN_HEIGHT);
            ctx.putImageData(imageData, 0, 0);

            // --- Draw the Tileset ---
            const tilesetPtr = emu.tileset_ptr();
            const tilesetPixelData = new Uint8ClampedArray(memory.buffer, tilesetPtr, TILESET_BUFFER_SIZE);
            const tilesetImageData = new ImageData(tilesetPixelData, TILESET_WIDTH, TILESET_HEIGHT);
            tilesetCtx.putImageData(tilesetImageData, 0, 0);
        }

        // Always request the next animation frame to keep the UI responsive.
        requestAnimationFrame(renderLoop);
    };

    // Start the emulation loop!
    requestAnimationFrame(renderLoop);
}

run().catch(console.error);