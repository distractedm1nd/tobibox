# TobiBox
Rust project for the Adafruit Feather RP2040, an RFID activated music box for children.

Also includes a module, yet to be separated into its own crate, for using the DFPlayer Mini MP3 player with embassy-rp.

## Usage
- Connect the Feather RP2040 to your computer via USB.
```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut tobibox = TobiBox::from_adafruit_feather(p, spawner);

    // "Dr. Seuss: Sleep Book" Read by Dad
    tobibox.register(&[51, 251, 160, 21], 1);
    // "Frosch Buch" Read by Mama
    tobibox.register(&[83, 202, 168, 21], 2);

    loop {
        tobibox.wupa();
        Timer::after_millis(200).await;
    }
}
```

## TODO
- df_player 0x41 ack/reading from rx
- df_player publish as own crate
