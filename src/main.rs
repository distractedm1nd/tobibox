#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals, bind_interrupts,
    gpio::{Level, Output},
    peripherals::{SPI0, USB},
    spi::{self, Blocking, Spi},
    uart,
    usb::{Driver, InterruptHandler as USBInterruptHandler},
};
use embassy_time::{Delay, Timer};
use log::{error, info};
use panic_probe as _;

use embedded_hal_bus::spi::ExclusiveDevice;

use mfrc522::{
    Initialized,
    comm::blocking::spi::{DummyDelay, SpiInterface},
};
use mfrc522::{Mfrc522, Uid};

mod df_player;

use defmt_rtt as _;
use panic_probe as _;

// Bind interrupts to their handlers.
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

type RFCDevice<'a> = Mfrc522<
    SpiInterface<ExclusiveDevice<Spi<'a, SPI0, Blocking>, Output<'a>, Delay>, DummyDelay>,
    Initialized,
>;

type Registry = heapless::LinearMap<[u8; 4], u16, 20>;

#[derive(Eq, PartialEq)]
enum PlayingState {
    Playing(u16),
    Asleep,
}

struct TobiBox<'a> {
    mrfc: RFCDevice<'a>,
    df_player: df_player::DFPlayerMini<'a>,
    registry: Registry,
    playing_state: PlayingState,
}

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

impl<'a> TobiBox<'a> {
    /// Creates a new TobiBox instance from an Adafruit Feather RP2040 board.
    /// Pin Layout:
    /// - PIN_18: SPI0_SCK   -> RC522 SCL
    /// - PIN_19: SPI0_MOSI  -> RC522 MOSI
    /// - PIN_20: SPI0_MISO  -> RC522 MISO
    /// - PIN_6:  SDA        -> RC522 SDA
    /// - PIN_0:  UART0_TX   -> DF_Player_Mini RX
    /// - PIN_1:  UART0_RX   -> DF_Player_Mini TX
    pub fn from_adafruit_feather(p: Peripherals, spawner: Spawner) -> Self {
        let usb_driver = Driver::new(p.USB, Irqs);

        spawner.spawn(logger_task(usb_driver)).unwrap();

        let mut spi_config = spi::Config::default();
        spi_config.frequency = 1_000_000;
        let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_20, spi_config);

        let cs = Output::new(p.PIN_6, Level::High);

        let spi = ExclusiveDevice::new(spi, cs, Delay);
        let itf = SpiInterface::new(spi);
        let mfrc522 = Mfrc522::new(itf).init().expect("could not create MFRC522");

        info!("Initializing df player");
        let mut config = uart::Config::default();
        config.baudrate = 9600;
        let uart = uart::Uart::new_blocking(p.UART0, p.PIN_0, p.PIN_1, config);
        let mut df_player = df_player::DFPlayerMini { usart: uart };

        info!("Calling Reset");
        df_player.write_command(df_player::Command::Reset);
        df_player.write_command(df_player::Command::SetVolume(0x05));

        Self::new(mfrc522, df_player)
    }

    pub fn new(mrfc: RFCDevice<'a>, df_player: df_player::DFPlayerMini<'a>) -> Self {
        Self {
            mrfc,
            df_player,
            registry: Registry::new(),
            playing_state: PlayingState::Asleep,
        }
    }

    pub fn register(&mut self, uid: &[u8; 4], song_id: u16) {
        self.registry
            .insert(*uid, song_id)
            .expect("Failed to register card");
    }

    /// Plays a song based on the registered UID and updates the playing state.
    fn handle_card(&mut self, uid: &Uid) {
        let song_id = self.registry.get(uid.as_bytes());
        if let Some(id) = song_id {
            if self.playing_state != PlayingState::Playing(*id) {
                self.playing_state = PlayingState::Playing(*id);
                self.df_player
                    .write_command(df_player::Command::PlayWithIndex(*id));
            }
        } else {
            self.playing_state = PlayingState::Asleep;
            self.df_player.write_command(df_player::Command::VolumeDown);
        }
    }

    /// Sends a Wake UP type A to nearby PICCs and handles any found Uids
    pub fn wupa(&mut self) {
        if let Ok(atqa) = self.mrfc.wupa() {
            info!("new card detected");
            match self.mrfc.select(&atqa) {
                Ok(ref uid @ Uid::Single(ref inner)) => {
                    info!("card uid {:?}", inner.as_bytes());
                    self.handle_card(uid);
                }
                Ok(_) => info!("got other uid size"),
                Err(e) => {
                    error!("Select error {:?}", e);
                }
            }
        }
    }
}
