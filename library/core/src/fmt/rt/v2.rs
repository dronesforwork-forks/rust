#![warn(warnings)]
#![allow(missing_debug_implementations)]

use crate::fmt::Formatter;
use crate::fmt::Opaque;
use crate::fmt::Result;

pub use super::v1::Alignment;

pub struct Arguments<'a> {
    cmds: *const EncodedCmd,
    args: *const &'a Opaque,
}

impl<'a> Arguments<'a> {
    pub const unsafe fn new(
        cmds: &'static [EncodedCmd],
        args: &'a [&'a Opaque],
    ) -> Self {
        Self {
            cmds: cmds.as_ptr(),
            args: args.as_ptr(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct EncodedCmd {
    pub ptr_or_val: usize,
    pub type_or_len: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Cmd {
    Literal(&'static str),
    SetWidth {
        width: usize,
    },
    SetPrecision {
        precision: usize,
    },
    SetWidthFromArg {
        arg_index: usize,
    },
    SetPrecisionFromArg {
        arg_index: usize,
    },
    SetFlags {
        filler: char,
        alignment: Alignment,
        flags: u8,
    },
    FormatArg {
        arg_index: usize,
        formatter: FormatterFn,
    },
    End,
}

#[derive(Copy, Clone)]
pub struct FormatterFn(fn(&Opaque, &mut Formatter<'_>) -> Result);

impl PartialEq for FormatterFn {
    fn eq(&self, other: &FormatterFn) -> bool {
        self.0 as usize == other.0 as usize
    }
}

impl crate::fmt::Debug for FormatterFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:p}", self.0 as *const ())
    }
}

const HIGH_1: usize = 0b1usize.reverse_bits();
const HIGH_11: usize = 0b11usize.reverse_bits();
const HIGH_101: usize = 0b101usize.reverse_bits();

impl Cmd {
    pub fn encode(self) -> EncodedCmd {
        match self {
            Cmd::Literal(s) => EncodedCmd {
                ptr_or_val: s.as_ptr() as usize,
                type_or_len: s.len(),
            },
            Cmd::End => EncodedCmd {
                ptr_or_val: 0,
                type_or_len: HIGH_1,
            },
            Cmd::SetWidth { width } => EncodedCmd {
                ptr_or_val: width,
                type_or_len: HIGH_1 | 2,
            },
            Cmd::SetPrecision { precision } => EncodedCmd {
                ptr_or_val: precision,
                type_or_len: HIGH_1 | 3,
            },
            Cmd::SetWidthFromArg { arg_index } => EncodedCmd {
                ptr_or_val: arg_index,
                type_or_len: HIGH_1 | 4,
            },
            Cmd::SetPrecisionFromArg { arg_index } => EncodedCmd {
                ptr_or_val: arg_index,
                type_or_len: HIGH_1 | 5,
            },
            Cmd::SetFlags {
                filler,
                alignment,
                flags,
            } => {
                let align = match alignment {
                    Alignment::Left => 0,
                    Alignment::Right => 1,
                    Alignment::Center => 2,
                    Alignment::Unknown => 3,
                };
                debug_assert_eq!(flags >> 6, 0);
                let flags = (flags | align << 6) as usize;
                EncodedCmd {
                    ptr_or_val: filler as usize,
                    type_or_len: if usize::BITS == 16 {
                        // The filler char doesn't fit entirely in the
                        // ptr_or_val on 16-bit platforms, so we put the five
                        // bits that don't fit into the type_or_len, where
                        // there happens to be space for exactly five bits.
                        HIGH_101 | flags << 5 | (filler as u32 >> 16) as usize
                    } else {
                        HIGH_101 | flags
                    },
                }
            }
            Cmd::FormatArg {
                arg_index,
                formatter,
            } => {
                // This is always true because an index cannot exceed
                // isize::MAX when counted in *bytes*, and an argument is at
                // least two bytes, since an usize is u16 or bigger.
                debug_assert!(arg_index <= !HIGH_11);
                EncodedCmd {
                    ptr_or_val: formatter.0 as usize,
                    type_or_len: HIGH_11 | arg_index,
                }
            }
        }
    }
}

impl EncodedCmd {
    pub fn decode(self) -> Cmd {
        match (self.ptr_or_val, self.type_or_len) {
            (ptr, len) if len < HIGH_1 => Cmd::Literal(unsafe {
                crate::str::from_utf8_unchecked(crate::slice::from_raw_parts(ptr as *const u8, len))
            }),
            (fn_ptr, cmd) if cmd >= HIGH_11 => Cmd::FormatArg {
                arg_index: cmd & !HIGH_11,
                formatter: FormatterFn(unsafe {
                    crate::mem::transmute(fn_ptr)
                }),
            },
            (arg, cmd) if cmd >= HIGH_101 => {
                let (filler, flags) = if usize::BITS == 16 {
                    (cmd as u32 & 0b1_1111 << 16 | arg as u32, (cmd >> 5) as u8)
                } else {
                    (arg as u32, cmd as u8)
                };
                let alignment = match flags >> 6 {
                    0 => Alignment::Left,
                    1 => Alignment::Right,
                    2 => Alignment::Center,
                    _ => Alignment::Unknown,
                };
                let flags = flags & 0b11_1111;
                Cmd::SetFlags {
                    filler: unsafe { char::from_u32_unchecked(filler) },
                    alignment,
                    flags,
                }
            }
            (_, cmd) if cmd == HIGH_1 => Cmd::End,
            (v, cmd) if cmd == HIGH_1 | 2 => Cmd::SetWidth { width: v },
            (v, cmd) if cmd == HIGH_1 | 3 => Cmd::SetPrecision { precision: v },
            (i, cmd) if cmd == HIGH_1 | 4 => Cmd::SetWidthFromArg { arg_index: i },
            (i, cmd) if cmd == HIGH_1 | 5 => Cmd::SetPrecisionFromArg { arg_index: i },
            _ => unsafe { crate::hint::unreachable_unchecked() },
        }
    }
}
