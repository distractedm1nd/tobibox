#![allow(dead_code)]

use embassy_rp::uart::{Blocking, Uart};

const START_BYTE: u8 = 0x7E;
const VERSION_BYTE: u8 = 0xFF;
const COMMAND_LENGTH: u8 = 0x06;
const END_BYTE: u8 = 0xEF;

///  Returns info with command 0x41 [0x01: info, 0x00: no info]
// TODO: Actually just use a bool in execute_command
const ACK: u8 = 0x00;

pub struct DFPlayerMini<'a> {
    pub usart: Uart<'a, Blocking>,
    // rcv_buffer: [u8; 10]
}

/// Used with [`Command::SetEQ`]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum EQ {
    Normal,
    Pop,
    Rock,
    Jazz,
    Classic,
    Base,
}

#[repr(u8)]
pub enum Command {
    /// Play next song
    NextSong = 0x01,
    /// Play previous song
    PrevSong = 0x02,
    /// Play with index
    PlayWithIndex(u16) = 0x03,
    /// Volume up
    VolumeUp = 0x04,
    /// Volume down
    VolumeDown = 0x05,
    /// Set volume (0-30)
    SetVolume(u16) = 0x06,
    /// SetEq
    SetEQ(EQ) = 0x07,
    /// Loop specific song
    LoopSong(u16) = 0x08,
    /// Select storage device to USB memory if true, else SD card
    SetDevice(bool) = 0x09,
    /// Chip enters sleep mode
    SleepMode = 0x0A,
    /// Chip enters sleep mode
    WakeUp = 0x0B,
    ///Chip reset
    Reset = 0x0C,
    /// Resume playback
    Play = 0x0D,
    /// Pause playback
    Pause = 0x0E,
    /// Play specific song in a folder (1st param) that supports 256 songs
    /// module suports 256 folders (0 - 255) with 255 songs.
    PlayFolder(u8, u8) = 0x0F,
    /// Audio amplification
    // 01 – Amp ON;
    // 00 – Amp OFF;
    // Level 0-31
    SetAmplification(u8, u8) = 0x10,
    /// Start/Stop looping all songs
    SetLoopAll(bool) = 0x11,
    /// Play song in mp3 folder
    /// (0x0001 – 0x0BB8; 3000 songs)
    PlayInMP3Folder(u16) = 0x12,
    /// Play advert
    /// (0x0001 – 0x0BB8; 3000 songs)
    PlayAdvert = 0x13,
    /// Stop playing advert and resume previous playback
    StopAdvert = 0x15,
    // Play specific song in the folder that supports 3000 songs; module suports
    // 16 folders (0 - 15) with 3000 songs.
    PlayInFolder2(u8, u8) = 0x14,
    /// Enable loop all
    // TODO: What is difference to SetLoopAll?
    EnableLoopAll(bool) = 0x16,
    LoopFromFolder(u8, u8) = 0x17,
    /// Random playback
    RandomPlayback = 0x18,
    /// Set single loop play
    SetSingleLoopPlay(bool) = 0x19,
    SetDAC(bool) = 0x1A,
    PlaySongWithVolume(u8, u8) = 0x22,
}

fn split(int: &u16) -> (u8, u8) {
    let bytes = int.to_be_bytes();
    (bytes[0], bytes[1])
}

impl Command {
    pub fn convert_with_params(&self) -> (u8, u8, u8) {
        let cmd_byte = self.cmd_byte();
        match self {
            // All with u16 parameter
            Self::PlayWithIndex(a) | Self::PlayInMP3Folder(a) | Self::SetVolume(a) => {
                let (p1, p2) = split(a);
                (cmd_byte, p1, p2)
            }
            // All with (u8, u8) parameters
            Self::PlayFolder(a, b)
            | Self::PlayInFolder2(a, b)
            | Self::LoopFromFolder(a, b)
            | Self::PlaySongWithVolume(a, b)
            | Self::SetAmplification(a, b) => (cmd_byte, *a, *b),
            // All with bool param
            Self::SetDevice(a)
            | Self::SetLoopAll(a)
            | Self::EnableLoopAll(a)
            | Self::SetSingleLoopPlay(a)
            | Self::SetDAC(a) => {
                let p2: u8 = match a {
                    true => 1,
                    false => 0,
                };
                (cmd_byte, 0, p2)
            }
            Self::SetEQ(eq) => (cmd_byte, 0, *eq as u8),
            _ => (cmd_byte, 0, 0),
        }
    }

    fn cmd_byte(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl DFPlayerMini<'_> {
    /// Sends command to module
    pub fn write_command(&mut self, cmd: Command) {
        let (cmd, p1, p2) = cmd.convert_with_params();
        // let checksum: i16 = -(VERSION_BYTE as i16
        //     + COMMAND_LENGTH as i16
        //     + cmd as i16
        //     + ACK as i16
        //     + p1 as i16
        //     + p2 as i16);
        // let cs_bytes = checksum.to_be_bytes();
        let out = &[
            START_BYTE,
            VERSION_BYTE,
            COMMAND_LENGTH,
            cmd,
            ACK,
            p1,
            p2,
            // cs_bytes[1],
            // cs_bytes[0],
            END_BYTE,
        ];

        self.usart.blocking_write(out).unwrap();
        //self.usart.blocking_flush().unwrap();
    }
}
