use crate::{console, exit, print, println};

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    'prompt: loop {
        print!("> ");

        let mut cmdline = [0u8; 128];

        for i in 0.. {
            let ch = console::getchar() as u8;
            console::putchar(ch);
            if i == cmdline.len() - 1 {
                println!("command line too long");
                continue 'prompt;
            } else if ch == b'\r' {
                println!();
                cmdline[i] = b'\0';
                break;
            } else {
                cmdline[i] = ch;
            }
        }

        unsafe {
            if common::strcmp(cmdline.as_ptr(), b"hello\0".as_ptr()) == 0 {
                println!("Hello world from shell!");
            } else if common::strcmp(cmdline.as_ptr(), b"exit\0".as_ptr()) == 0 {
                exit();
            } else {
                print!("unknown command: ");

                for ch in cmdline {
                    console::putchar(ch);
                }

                println!();
            }
        }
    }
}
