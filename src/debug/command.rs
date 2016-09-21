use std::borrow::{Cow};
use std::str;
use std::str::{FromStr};

use nom::{IResult, digit, eof, space};

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Exit,
    Step(usize),
    Repeat
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match command(s.as_bytes()) {
            IResult::Done(_, c) => Ok(c),
            err => Err(format!("Unrecognised command: {:?}", err).into()),
        }
    }
}

named!(
    command<Command>,
    chain!(
        c: alt_complete!(
            step |
            exit |
            repeat
        ) ~
        eof ,
        || c
    )
);

named!(
    step<Command>,
    chain!(
        alt_complete!(tag!("step") | tag!("s")) ~
        count: opt!(preceded!(space, usize_parser)),

        || Command::Step(count.unwrap_or(1))
    )
);

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("quit") | tag!("e") | tag!("q")),
        |_| Command::Exit
    )
);

named!(
    repeat<Command>,
    value!(Command::Repeat)
);

named!(
    usize_parser<usize>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8
        ),
        FromStr::from_str
    )
);