use super::{Uart, STDOUT};
use core::fmt::{Arguments, Result, Write};

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        let mut buffer = [0u8; 4];
        let mut stdout = STDOUT.lock();
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                stdout.put(*code_point);
            }
        }
        Ok(())
    }
}

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn __print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}

use preprint::Print;

pub struct PrePrint;

impl Print for PrePrint {
    fn print(&self, args: Arguments) {
        print!("{}", args);
    }
}
impl Write for PrePrint {
    fn write_str(&mut self, s: &str) -> Result {
        print!("{}", s);
        Ok(())
    }
}
