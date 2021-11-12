// automatically generated by the FlatBuffers compiler, do not modify

use std::cmp::Ordering;
use std::mem;

extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};

#[allow(unused_imports, dead_code)]
pub mod mcfs {

    use std::cmp::Ordering;
    use std::mem;

    extern crate flatbuffers;
    use self::flatbuffers::{EndianScalar, Follow};

    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    pub const ENUM_MIN_COMMAND_TYPE: i32 = 0;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    pub const ENUM_MAX_COMMAND_TYPE: i32 = 2;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    #[allow(non_camel_case_types)]
    pub const ENUM_VALUES_COMMAND_TYPE: [CommandType; 3] = [
        CommandType::PlayerHealth,
        CommandType::PlayerName,
        CommandType::PlayerPosition,
    ];

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct CommandType(pub i32);
    #[allow(non_upper_case_globals)]
    impl CommandType {
        pub const PlayerHealth: Self = Self(0);
        pub const PlayerName: Self = Self(1);
        pub const PlayerPosition: Self = Self(2);

        pub const ENUM_MIN: i32 = 0;
        pub const ENUM_MAX: i32 = 2;
        pub const ENUM_VALUES: &'static [Self] =
            &[Self::PlayerHealth, Self::PlayerName, Self::PlayerPosition];
        /// Returns the variant's name or "" if unknown.
        pub fn variant_name(self) -> Option<&'static str> {
            match self {
                Self::PlayerHealth => Some("PlayerHealth"),
                Self::PlayerName => Some("PlayerName"),
                Self::PlayerPosition => Some("PlayerPosition"),
                _ => None,
            }
        }
    }
    impl std::fmt::Debug for CommandType {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            if let Some(name) = self.variant_name() {
                f.write_str(name)
            } else {
                f.write_fmt(format_args!("<UNKNOWN {:?}>", self.0))
            }
        }
    }
    impl<'a> flatbuffers::Follow<'a> for CommandType {
        type Inner = Self;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            let b = unsafe { flatbuffers::read_scalar_at::<i32>(buf, loc) };
            Self(b)
        }
    }

    impl flatbuffers::Push for CommandType {
        type Output = CommandType;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            unsafe {
                flatbuffers::emplace_scalar::<i32>(dst, self.0);
            }
        }
    }

    impl flatbuffers::EndianScalar for CommandType {
        #[inline]
        fn to_little_endian(self) -> Self {
            let b = i32::to_le(self.0);
            Self(b)
        }
        #[inline]
        #[allow(clippy::wrong_self_convention)]
        fn from_little_endian(self) -> Self {
            let b = i32::from_le(self.0);
            Self(b)
        }
    }

    impl<'a> flatbuffers::Verifiable for CommandType {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            i32::run_verifier(v, pos)
        }
    }

    impl flatbuffers::SimpleToVerifyInSlice for CommandType {}
    pub enum CommandOffset {}
    #[derive(Copy, Clone, PartialEq)]

    pub struct Command<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for Command<'a> {
        type Inner = Command<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf, loc },
            }
        }
    }

    impl<'a> Command<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            Command { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args CommandArgs,
        ) -> flatbuffers::WIPOffset<Command<'bldr>> {
            let mut builder = CommandBuilder::new(_fbb);
            builder.add_cmd(args.cmd);
            builder.finish()
        }

        pub const VT_CMD: flatbuffers::VOffsetT = 4;

        #[inline]
        pub fn cmd(&self) -> CommandType {
            self._tab
                .get::<CommandType>(Command::VT_CMD, Some(CommandType::PlayerHealth))
                .unwrap()
        }
    }

    impl flatbuffers::Verifiable for Command<'_> {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            v.visit_table(pos)?
                .visit_field::<CommandType>(&"cmd", Self::VT_CMD, false)?
                .finish();
            Ok(())
        }
    }
    pub struct CommandArgs {
        pub cmd: CommandType,
    }
    impl<'a> Default for CommandArgs {
        #[inline]
        fn default() -> Self {
            CommandArgs {
                cmd: CommandType::PlayerHealth,
            }
        }
    }
    pub struct CommandBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> CommandBuilder<'a, 'b> {
        #[inline]
        pub fn add_cmd(&mut self, cmd: CommandType) {
            self.fbb_
                .push_slot::<CommandType>(Command::VT_CMD, cmd, CommandType::PlayerHealth);
        }
        #[inline]
        pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CommandBuilder<'a, 'b> {
            let start = _fbb.start_table();
            CommandBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<Command<'a>> {
            let o = self.fbb_.end_table(self.start_);
            flatbuffers::WIPOffset::new(o.value())
        }
    }

    impl std::fmt::Debug for Command<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut ds = f.debug_struct("Command");
            ds.field("cmd", &self.cmd());
            ds.finish()
        }
    }
    #[inline]
    #[deprecated(since = "2.0.0", note = "Deprecated in favor of `root_as...` methods.")]
    pub fn get_root_as_command<'a>(buf: &'a [u8]) -> Command<'a> {
        unsafe { flatbuffers::root_unchecked::<Command<'a>>(buf) }
    }

    #[inline]
    #[deprecated(since = "2.0.0", note = "Deprecated in favor of `root_as...` methods.")]
    pub fn get_size_prefixed_root_as_command<'a>(buf: &'a [u8]) -> Command<'a> {
        unsafe { flatbuffers::size_prefixed_root_unchecked::<Command<'a>>(buf) }
    }

    #[inline]
    /// Verifies that a buffer of bytes contains a `Command`
    /// and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_command_unchecked`.
    pub fn root_as_command(buf: &[u8]) -> Result<Command, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::root::<Command>(buf)
    }
    #[inline]
    /// Verifies that a buffer of bytes contains a size prefixed
    /// `Command` and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `size_prefixed_root_as_command_unchecked`.
    pub fn size_prefixed_root_as_command(
        buf: &[u8],
    ) -> Result<Command, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::size_prefixed_root::<Command>(buf)
    }
    #[inline]
    /// Verifies, with the given options, that a buffer of bytes
    /// contains a `Command` and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_command_unchecked`.
    pub fn root_as_command_with_opts<'b, 'o>(
        opts: &'o flatbuffers::VerifierOptions,
        buf: &'b [u8],
    ) -> Result<Command<'b>, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::root_with_opts::<Command<'b>>(opts, buf)
    }
    #[inline]
    /// Verifies, with the given verifier options, that a buffer of
    /// bytes contains a size prefixed `Command` and returns
    /// it. Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_command_unchecked`.
    pub fn size_prefixed_root_as_command_with_opts<'b, 'o>(
        opts: &'o flatbuffers::VerifierOptions,
        buf: &'b [u8],
    ) -> Result<Command<'b>, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::size_prefixed_root_with_opts::<Command<'b>>(opts, buf)
    }
    #[inline]
    /// Assumes, without verification, that a buffer of bytes contains a Command and returns it.
    /// # Safety
    /// Callers must trust the given bytes do indeed contain a valid `Command`.
    pub unsafe fn root_as_command_unchecked(buf: &[u8]) -> Command {
        flatbuffers::root_unchecked::<Command>(buf)
    }
    #[inline]
    /// Assumes, without verification, that a buffer of bytes contains a size prefixed Command and returns it.
    /// # Safety
    /// Callers must trust the given bytes do indeed contain a valid size prefixed `Command`.
    pub unsafe fn size_prefixed_root_as_command_unchecked(buf: &[u8]) -> Command {
        flatbuffers::size_prefixed_root_unchecked::<Command>(buf)
    }
    #[inline]
    pub fn finish_command_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<Command<'a>>,
    ) {
        fbb.finish(root, None);
    }

    #[inline]
    pub fn finish_size_prefixed_command_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<Command<'a>>,
    ) {
        fbb.finish_size_prefixed(root, None);
    }
} // pub mod MCFS
