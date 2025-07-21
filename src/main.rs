#![no_std]
#![no_main]

use arduino_hal::{hal::{usart::BaudrateArduinoExt}};
use panic_halt as _;

mod df_player;
use df_player::{Command, DFPlayerMini};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    
    let pins = arduino_hal::pins!(dp);
    let (rx, tx) = (pins.d0, pins.d1.into_output());
    
    let usart = arduino_hal::Usart::new(dp.USART0, rx, tx, 9600.into_baudrate());
    let mut mp3_player = DFPlayerMini{
        usart
    };
    
    arduino_hal::delay_ms(1000);
    mp3_player.write_command(Command::SetVolume(20));
    mp3_player.write_command(Command::SetAmplification(1, 15));
    mp3_player.write_command(Command::RandomPlayback);
    
    loop {
        mp3_player.write_command(Command::Play);
        arduino_hal::delay_ms(10000);
        mp3_player.write_command(Command::Pause);
        arduino_hal::delay_ms(10000);
    }
}
