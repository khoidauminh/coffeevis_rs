pub const pif32 : f32 = std::f32::consts::PI;
pub const pih : f32 = pif32/2.0;
pub const pi2 : f32 = pif32*2.0;

pub const FPS : u64 = 72;
pub const FPS_ITVL : u64 = 1_000_000/FPS;
pub const CTRL_ITVL : u64 = FPS << 2;

pub const PHASE_OFFSET : usize = 2000;
pub static mut WAV_WIN : usize = 30; // in percentage;

pub const INCREMENT : usize  = 8;
pub static mut VOL_SCL : f32 = 0.95; // in percentage

pub const POWER : usize = 12;
pub const FFT_SIZE : usize = 1 << POWER;

pub const WIN_W : usize = 144;
pub const WIN_H : usize = 144;

pub static mut SMOOTHING : f32 = 0.5;

pub static mut EXIT : bool = false;

pub const VISES : usize = 8;
pub static mut VIS_IDX : usize = 0;
pub static mut SWITCH : usize = 0;

pub static mut MESSAGE : String = String::new();
pub static mut MESSAGE_TIMEOUT : u64 = 0; // timeout is FPS.

// pub static mut cos_table : [f32; 2048] = [0.0f32; 2048];

// pub unsafe fn prepare_table() {
//     let mut i : usize = 0;

//     while i < 2048 {
//         cos_table[i] = (i as f32 / 2048.0 * pi2).cos();
//         i += 1;
//     }
// }

//pub fn message_vol =

pub fn console_clear() {
    print!("\x1B[2J\x1B[H");
}

pub fn usleep(micros : u64) {
    std::thread::sleep(std::time::Duration::from_micros(micros));
}
