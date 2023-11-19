use base::input::KEYBOARD_QUEUE;

use crate::{
    disable_cursor,
    display::{
        restore_text_mode_font, switch_graphics_mode, BitmapVgaWriter, VgaModeSwitch, VgaPalette,
        VgaPaletteColor, STATIC_VGA_WRITER,
    },
    interrupt::setup::global_os_time,
};

pub fn show_lars() -> bool {
    let mut skipped = false;
    let timestamp = unsafe { global_os_time };
    let mut g_formatter = unsafe {
        switch_graphics_mode(VgaModeSwitch::VGA_320X200_BITMAP_N);
        BitmapVgaWriter::new_unsafe()
    };
    let mut fade = 0u8;
    let lars = include_bytes!("LarsKagg2.bmp");
    let width = i32::from_le_bytes(core::array::from_fn(|i| lars[i + 0x12]));
    let height = i32::from_le_bytes(core::array::from_fn(|i| lars[i + 0x16]));
    let palette = VgaPalette::from_array(core::array::from_fn(|i| {
        let chunk = &lars[0x36 + (i * 4)..0x36 + ((i + 1) * 4)];
        VgaPaletteColor::from_rgb(chunk[2], chunk[1], chunk[0])
    }));
    g_formatter.set_palette(VgaPalette::ALL_BLACK);
    let x_pos = (320 - width) / 2;
    for i in 0..height {
        for j in 0..width {
            let (x, y) = (x_pos as usize + j as usize, height as usize - i as usize - 1);
            let char = lars[0x436 + (j as usize + (i as usize * width as usize))];
            g_formatter.plot_pixel(x, y, char);
        }
    }
    let duration = 3000;
    let total_range = 0..duration;
    let visible_range = 1000..2000;
    while unsafe { global_os_time } < timestamp + duration {
        if let Some('w') = unsafe { KEYBOARD_QUEUE.try_getch() } {
            skipped = true;
            break;
        }
        let time = unsafe { global_os_time / 10 } as u8;
        for line in 196..200 {
            g_formatter.set_position((0, line));
            for i in 0..160u16 {
                g_formatter.write_char(time + i as u8);
                g_formatter.write_char(time + i as u8);
            }
        }

        let old_fade = fade;
        let time = unsafe { global_os_time } - timestamp;
        if !visible_range.contains(&time) {
            if time < visible_range.start {
                fade = ((time - total_range.start) * u8::MAX as u64
                    / (visible_range.start - total_range.start)) as u8;
            }
            if time >= visible_range.end {
                fade = u8::MAX
                    - ((time - visible_range.end) * u8::MAX as u64
                        / (total_range.end - visible_range.end)) as u8;
            }
        }
        if old_fade != fade {
            g_formatter.set_palette(palette.fade_factor(fade))
        }
    }
    switch_graphics_mode(VgaModeSwitch::VGA_80X25_TEXT);
    disable_cursor();
    unsafe {
        restore_text_mode_font();
        STATIC_VGA_WRITER.clear_screen(crate::display::VgaColor::Black);
    }
    skipped
}
