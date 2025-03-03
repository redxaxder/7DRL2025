

use macroquad::prelude::*;

#[macroquad::main("7drl")]
async fn main() {
    debug!("This is a debug message");
    info!("and info message");
    error!("and errors, the red ones!");
    warn!("Or warnings, the yellow ones.");

    loop {
        clear_background(LIGHTGRAY);

        debug!("Still alive!");

        next_frame().await
    }
}


