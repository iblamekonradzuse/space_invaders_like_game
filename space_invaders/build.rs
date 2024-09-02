use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("../../../audio");

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(&dest_path).expect("Failed to create audio directory");

    // Copy audio files
    let audio_files = ["background.mp3", "laser.mp3"];
    for file in &audio_files {
        let src = format!("src/audio/{}", file);
        let dest = dest_path.join(file);

        // Only copy if the source file exists and is different from the destination
        if Path::new(&src).exists()
            && (!dest.exists()
                || fs::metadata(&src).unwrap().modified().unwrap()
                    != fs::metadata(&dest).unwrap().modified().unwrap())
        {
            fs::copy(&src, &dest).expect(&format!("Failed to copy {}", file));
        }
    }

    println!("cargo:rerun-if-changed=src/audio");
}

