mod cpu;
mod gamepak;
mod gba;
mod mem;

fn main() {
    //let rom_path = "test/roms/240pee_mb.gba";
    let rom_path = "/home/aphistic/Downloads/pokemon-sapphire.gba";


    let gp = match gamepak::GamePak::load_from_file(rom_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Error loading gamepak: {}", e);
            return;
        }
    };

    println!("game title: {} ({})", gp.header().game_title(), gp.header().game_title().len());
    println!("game code: {} ({})", gp.header().game_code(), gp.header().game_code().len());
    println!("maker code: {}", gp.header().maker_code());

    let mut console = gba::GBA::new();
    match console.load(gp) {
        Ok(_) => println!("loaded console"),
        Err(e) => println!("could not load console: {}", e)
    }

    println!("stepping");
    console.step()
}
