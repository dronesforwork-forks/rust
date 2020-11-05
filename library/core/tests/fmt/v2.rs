use core::fmt::rt::v2::{Alignment, Cmd};

#[test]
fn fmt_cmd_encode_test() {
    for &cmd in &[
        Cmd::Literal(""),
        Cmd::Literal("hello"),
        Cmd::SetWidth { width: 9 },
        Cmd::SetPrecision { precision: 100 },
        Cmd::SetWidthFromArg { arg_index: 1 },
        Cmd::SetPrecisionFromArg { arg_index: 3 },
        Cmd::End,
        Cmd::SetFlags {
            filler: '-',
            flags: 0,
            alignment: Alignment::Left,
        },
        Cmd::SetFlags {
            filler: char::MAX,
            flags: 0b10_1011,
            alignment: Alignment::Center,
        },
        Cmd::FormatArg {
            arg_index: 0,
            formatter: unsafe { core::mem::transmute(8usize) },
        },
        Cmd::FormatArg {
            arg_index: 9,
            formatter: unsafe { core::mem::transmute(0xbbaausize.swap_bytes() + 0xddee) },
        },
        Cmd::FormatArg {
            arg_index: usize::MAX >> 2,
            formatter: unsafe { core::mem::transmute(0x1234usize) },
        },
    ] {
        assert_eq!(cmd.encode().decode(), cmd);
    }
}
