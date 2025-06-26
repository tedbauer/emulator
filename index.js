// We will only import the default `init` function.
import init from './pkg/emulator.js';

// --- DEBUGGING LOGS ---
console.log("index.js script started.");

const SCREEN_WIDTH = 160;
const SCREEN_HEIGHT = 144;
const FRAMEBUFFER_SIZE = SCREEN_WIDTH * SCREEN_HEIGHT * 4; // RGBA = 4 bytes per pixel

const canvas = document.getElementById('emulator-canvas');
const ctx = canvas.getContext('2d');

// This main function now runs as soon as the page loads.
async function run() {
    // --- DEBUGGING LOGS ---
    console.log("run() function called. Awaiting init()...");

    // Initialize the WASM module. The object contains our exported functions.
    const wasmModule = await init();

    // --- DEBUGGING LOGS ---
    console.log("init() successful. WASM module loaded:", wasmModule);

    // Instead of a class, we call the exported `emulator_new` function.
    // This returns a "handle" or pointer to our Rust `Emulator` struct instance.
    const emu = wasmModule.emulator_new();
    const memory = wasmModule.memory; // Get the memory export

    // Wire up keyboard input to the emulator
    document.addEventListener('keydown', event => {
        // Pass the emulator instance handle to the function
        wasmModule.emulator_key_down(emu, event.code);
    });
    document.addEventListener('keyup', event => {
        // Pass the emulator instance handle to the function
        wasmModule.emulator_key_up(emu, event.code);
    });

    // The main render loop, driven by the browser
    const renderLoop = () => {
        // Pass the emulator instance handle to the tick function
        wasmModule.emulator_tick(emu);

        // Get the memory address of the framebuffer
        const framebufferPtr = wasmModule.emulator_framebuffer_ptr(emu);

        try {
            // This part remains the same. We create a view into the WASM memory.
            const pixelData = new Uint8ClampedArray(
                memory.buffer,
                framebufferPtr,
                FRAMEBUFFER_SIZE
            );

            // Create an ImageData object and paint it to the canvas
            const imageData = new ImageData(pixelData, SCREEN_WIDTH, SCREEN_HEIGHT);
            ctx.putImageData(imageData, 0, 0);
        } catch (e) {
            console.error("Error creating or drawing framebuffer:", e);
            // Stop the loop if there's an error to avoid flooding the console
            return;
        }

        // Schedule the next frame to be drawn
        requestAnimationFrame(renderLoop);
    };

    // Start the emulation loop!
    requestAnimationFrame(renderLoop);
}

// Run the emulator as soon as the page loads.
run().catch(e => {
    // --- DEBUGGING LOGS ---
    // Make sure we see any errors that happen during the run() function.
    console.error("Error running emulator:", e);
});
