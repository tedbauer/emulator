use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: gbsc <file.gbs> [-o output.gb]");
        process::exit(1);
    }

    let input = &args[1];
    let output = if let Some(p) = args.iter().position(|a| a == "-o") {
        args.get(p + 1).map(|s| s.as_str()).unwrap_or("out.gb")
    } else {
        "out.gb"
    };

    let src = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", input, e);
            process::exit(1);
        }
    };

    match compiler::compile(&src) {
        Ok(rom) => {
            if let Err(e) = std::fs::write(output, &rom) {
                eprintln!("Error writing '{}': {}", output, e);
                process::exit(1);
            }
            println!("✓ Compiled {} → {} ({} bytes)", input, output, rom.len());
        }
        Err(e) => {
            eprintln!("Compile error: {}", e);
            process::exit(1);
        }
    }
}
