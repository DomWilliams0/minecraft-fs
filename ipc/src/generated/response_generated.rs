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
    pub const ENUM_MIN_ERROR: i32 = 0;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    pub const ENUM_MAX_ERROR: i32 = 2;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    #[allow(non_camel_case_types)]
    pub const ENUM_VALUES_ERROR: [Error; 3] =
        [Error::Unknown, Error::UnknownCommand, Error::NoGame];

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct Error(pub i32);
    #[allow(non_upper_case_globals)]
    impl Error {
        pub const Unknown: Self = Self(0);
        pub const UnknownCommand: Self = Self(1);
        pub const NoGame: Self = Self(2);

        pub const ENUM_MIN: i32 = 0;
        pub const ENUM_MAX: i32 = 2;
        pub const ENUM_VALUES: &'static [Self] =
            &[Self::Unknown, Self::UnknownCommand, Self::NoGame];
        /// Returns the variant's name or "" if unknown.
        pub fn variant_name(self) -> Option<&'static str> {
            match self {
                Self::Unknown => Some("Unknown"),
                Self::UnknownCommand => Some("UnknownCommand"),
                Self::NoGame => Some("NoGame"),
                _ => None,
            }
        }
    }
    impl std::fmt::Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            if let Some(name) = self.variant_name() {
                f.write_str(name)
            } else {
                f.write_fmt(format_args!("<UNKNOWN {:?}>", self.0))
            }
        }
    }
    impl<'a> flatbuffers::Follow<'a> for Error {
        type Inner = Self;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            let b = unsafe { flatbuffers::read_scalar_at::<i32>(buf, loc) };
            Self(b)
        }
    }

    impl flatbuffers::Push for Error {
        type Output = Error;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            unsafe {
                flatbuffers::emplace_scalar::<i32>(dst, self.0);
            }
        }
    }

    impl flatbuffers::EndianScalar for Error {
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

    impl<'a> flatbuffers::Verifiable for Error {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            i32::run_verifier(v, pos)
        }
    }

    impl flatbuffers::SimpleToVerifyInSlice for Error {}
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    pub const ENUM_MIN_GAME_RESPONSE_BODY: u8 = 0;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    pub const ENUM_MAX_GAME_RESPONSE_BODY: u8 = 2;
    #[deprecated(
        since = "2.0.0",
        note = "Use associated constants instead. This will no longer be generated in 2021."
    )]
    #[allow(non_camel_case_types)]
    pub const ENUM_VALUES_GAME_RESPONSE_BODY: [GameResponseBody; 3] = [
        GameResponseBody::NONE,
        GameResponseBody::Response,
        GameResponseBody::StateResponse,
    ];

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct GameResponseBody(pub u8);
    #[allow(non_upper_case_globals)]
    impl GameResponseBody {
        pub const NONE: Self = Self(0);
        pub const Response: Self = Self(1);
        pub const StateResponse: Self = Self(2);

        pub const ENUM_MIN: u8 = 0;
        pub const ENUM_MAX: u8 = 2;
        pub const ENUM_VALUES: &'static [Self] = &[Self::NONE, Self::Response, Self::StateResponse];
        /// Returns the variant's name or "" if unknown.
        pub fn variant_name(self) -> Option<&'static str> {
            match self {
                Self::NONE => Some("NONE"),
                Self::Response => Some("Response"),
                Self::StateResponse => Some("StateResponse"),
                _ => None,
            }
        }
    }
    impl std::fmt::Debug for GameResponseBody {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            if let Some(name) = self.variant_name() {
                f.write_str(name)
            } else {
                f.write_fmt(format_args!("<UNKNOWN {:?}>", self.0))
            }
        }
    }
    impl<'a> flatbuffers::Follow<'a> for GameResponseBody {
        type Inner = Self;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            let b = unsafe { flatbuffers::read_scalar_at::<u8>(buf, loc) };
            Self(b)
        }
    }

    impl flatbuffers::Push for GameResponseBody {
        type Output = GameResponseBody;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            unsafe {
                flatbuffers::emplace_scalar::<u8>(dst, self.0);
            }
        }
    }

    impl flatbuffers::EndianScalar for GameResponseBody {
        #[inline]
        fn to_little_endian(self) -> Self {
            let b = u8::to_le(self.0);
            Self(b)
        }
        #[inline]
        #[allow(clippy::wrong_self_convention)]
        fn from_little_endian(self) -> Self {
            let b = u8::from_le(self.0);
            Self(b)
        }
    }

    impl<'a> flatbuffers::Verifiable for GameResponseBody {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            u8::run_verifier(v, pos)
        }
    }

    impl flatbuffers::SimpleToVerifyInSlice for GameResponseBody {}
    pub struct GameResponseBodyUnionTableOffset {}

    // struct Vec3, aligned to 8
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq)]
    pub struct Vec3(pub [u8; 24]);
    impl Default for Vec3 {
        fn default() -> Self {
            Self([0; 24])
        }
    }
    impl std::fmt::Debug for Vec3 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_struct("Vec3")
                .field("x", &self.x())
                .field("y", &self.y())
                .field("z", &self.z())
                .finish()
        }
    }

    impl flatbuffers::SimpleToVerifyInSlice for Vec3 {}
    impl flatbuffers::SafeSliceAccess for Vec3 {}
    impl<'a> flatbuffers::Follow<'a> for Vec3 {
        type Inner = &'a Vec3;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a Vec3>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a Vec3 {
        type Inner = &'a Vec3;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<Vec3>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for Vec3 {
        type Output = Vec3;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const Vec3 as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b Vec3 {
        type Output = Vec3;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const Vec3 as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl<'a> flatbuffers::Verifiable for Vec3 {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            v.in_buffer::<Self>(pos)
        }
    }
    impl<'a> Vec3 {
        #[allow(clippy::too_many_arguments)]
        pub fn new(x: f64, y: f64, z: f64) -> Self {
            let mut s = Self([0; 24]);
            s.set_x(x);
            s.set_y(y);
            s.set_z(z);
            s
        }

        pub fn x(&self) -> f64 {
            let mut mem = core::mem::MaybeUninit::<f64>::uninit();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.0[0..].as_ptr(),
                    mem.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<f64>(),
                );
                mem.assume_init()
            }
            .from_little_endian()
        }

        pub fn set_x(&mut self, x: f64) {
            let x_le = x.to_little_endian();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &x_le as *const f64 as *const u8,
                    self.0[0..].as_mut_ptr(),
                    core::mem::size_of::<f64>(),
                );
            }
        }

        pub fn y(&self) -> f64 {
            let mut mem = core::mem::MaybeUninit::<f64>::uninit();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.0[8..].as_ptr(),
                    mem.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<f64>(),
                );
                mem.assume_init()
            }
            .from_little_endian()
        }

        pub fn set_y(&mut self, x: f64) {
            let x_le = x.to_little_endian();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &x_le as *const f64 as *const u8,
                    self.0[8..].as_mut_ptr(),
                    core::mem::size_of::<f64>(),
                );
            }
        }

        pub fn z(&self) -> f64 {
            let mut mem = core::mem::MaybeUninit::<f64>::uninit();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.0[16..].as_ptr(),
                    mem.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<f64>(),
                );
                mem.assume_init()
            }
            .from_little_endian()
        }

        pub fn set_z(&mut self, x: f64) {
            let x_le = x.to_little_endian();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &x_le as *const f64 as *const u8,
                    self.0[16..].as_mut_ptr(),
                    core::mem::size_of::<f64>(),
                );
            }
        }
    }

    pub enum ResponseOffset {}
    #[derive(Copy, Clone, PartialEq)]

    pub struct Response<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for Response<'a> {
        type Inner = Response<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf, loc },
            }
        }
    }

    impl<'a> Response<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            Response { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args ResponseArgs<'args>,
        ) -> flatbuffers::WIPOffset<Response<'bldr>> {
            let mut builder = ResponseBuilder::new(_fbb);
            if let Some(x) = args.pos {
                builder.add_pos(x);
            }
            if let Some(x) = args.string {
                builder.add_string(x);
            }
            if let Some(x) = args.int {
                builder.add_int(x);
            }
            if let Some(x) = args.float {
                builder.add_float(x);
            }
            if let Some(x) = args.error {
                builder.add_error(x);
            }
            builder.finish()
        }

        pub const VT_ERROR: flatbuffers::VOffsetT = 4;
        pub const VT_FLOAT: flatbuffers::VOffsetT = 6;
        pub const VT_INT: flatbuffers::VOffsetT = 8;
        pub const VT_STRING: flatbuffers::VOffsetT = 10;
        pub const VT_POS: flatbuffers::VOffsetT = 12;

        #[inline]
        pub fn error(&self) -> Option<Error> {
            self._tab.get::<Error>(Response::VT_ERROR, None)
        }
        #[inline]
        pub fn float(&self) -> Option<f32> {
            self._tab.get::<f32>(Response::VT_FLOAT, None)
        }
        #[inline]
        pub fn int(&self) -> Option<i32> {
            self._tab.get::<i32>(Response::VT_INT, None)
        }
        #[inline]
        pub fn string(&self) -> Option<&'a str> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Response::VT_STRING, None)
        }
        #[inline]
        pub fn pos(&self) -> Option<&'a Vec3> {
            self._tab.get::<Vec3>(Response::VT_POS, None)
        }
    }

    impl flatbuffers::Verifiable for Response<'_> {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            v.visit_table(pos)?
                .visit_field::<Error>(&"error", Self::VT_ERROR, false)?
                .visit_field::<f32>(&"float", Self::VT_FLOAT, false)?
                .visit_field::<i32>(&"int", Self::VT_INT, false)?
                .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                    &"string",
                    Self::VT_STRING,
                    false,
                )?
                .visit_field::<Vec3>(&"pos", Self::VT_POS, false)?
                .finish();
            Ok(())
        }
    }
    pub struct ResponseArgs<'a> {
        pub error: Option<Error>,
        pub float: Option<f32>,
        pub int: Option<i32>,
        pub string: Option<flatbuffers::WIPOffset<&'a str>>,
        pub pos: Option<&'a Vec3>,
    }
    impl<'a> Default for ResponseArgs<'a> {
        #[inline]
        fn default() -> Self {
            ResponseArgs {
                error: None,
                float: None,
                int: None,
                string: None,
                pos: None,
            }
        }
    }
    pub struct ResponseBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> ResponseBuilder<'a, 'b> {
        #[inline]
        pub fn add_error(&mut self, error: Error) {
            self.fbb_
                .push_slot_always::<Error>(Response::VT_ERROR, error);
        }
        #[inline]
        pub fn add_float(&mut self, float: f32) {
            self.fbb_.push_slot_always::<f32>(Response::VT_FLOAT, float);
        }
        #[inline]
        pub fn add_int(&mut self, int: i32) {
            self.fbb_.push_slot_always::<i32>(Response::VT_INT, int);
        }
        #[inline]
        pub fn add_string(&mut self, string: flatbuffers::WIPOffset<&'b str>) {
            self.fbb_
                .push_slot_always::<flatbuffers::WIPOffset<_>>(Response::VT_STRING, string);
        }
        #[inline]
        pub fn add_pos(&mut self, pos: &Vec3) {
            self.fbb_.push_slot_always::<&Vec3>(Response::VT_POS, pos);
        }
        #[inline]
        pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ResponseBuilder<'a, 'b> {
            let start = _fbb.start_table();
            ResponseBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<Response<'a>> {
            let o = self.fbb_.end_table(self.start_);
            flatbuffers::WIPOffset::new(o.value())
        }
    }

    impl std::fmt::Debug for Response<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut ds = f.debug_struct("Response");
            ds.field("error", &self.error());
            ds.field("float", &self.float());
            ds.field("int", &self.int());
            ds.field("string", &self.string());
            ds.field("pos", &self.pos());
            ds.finish()
        }
    }
    pub enum StateResponseOffset {}
    #[derive(Copy, Clone, PartialEq)]

    pub struct StateResponse<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for StateResponse<'a> {
        type Inner = StateResponse<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf, loc },
            }
        }
    }

    impl<'a> StateResponse<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            StateResponse { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args StateResponseArgs,
        ) -> flatbuffers::WIPOffset<StateResponse<'bldr>> {
            let mut builder = StateResponseBuilder::new(_fbb);
            builder.add_is_in_game(args.is_in_game);
            builder.finish()
        }

        pub const VT_IS_IN_GAME: flatbuffers::VOffsetT = 4;

        #[inline]
        pub fn is_in_game(&self) -> bool {
            self._tab
                .get::<bool>(StateResponse::VT_IS_IN_GAME, Some(false))
                .unwrap()
        }
    }

    impl flatbuffers::Verifiable for StateResponse<'_> {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            v.visit_table(pos)?
                .visit_field::<bool>(&"is_in_game", Self::VT_IS_IN_GAME, false)?
                .finish();
            Ok(())
        }
    }
    pub struct StateResponseArgs {
        pub is_in_game: bool,
    }
    impl<'a> Default for StateResponseArgs {
        #[inline]
        fn default() -> Self {
            StateResponseArgs { is_in_game: false }
        }
    }
    pub struct StateResponseBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> StateResponseBuilder<'a, 'b> {
        #[inline]
        pub fn add_is_in_game(&mut self, is_in_game: bool) {
            self.fbb_
                .push_slot::<bool>(StateResponse::VT_IS_IN_GAME, is_in_game, false);
        }
        #[inline]
        pub fn new(
            _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        ) -> StateResponseBuilder<'a, 'b> {
            let start = _fbb.start_table();
            StateResponseBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<StateResponse<'a>> {
            let o = self.fbb_.end_table(self.start_);
            flatbuffers::WIPOffset::new(o.value())
        }
    }

    impl std::fmt::Debug for StateResponse<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut ds = f.debug_struct("StateResponse");
            ds.field("is_in_game", &self.is_in_game());
            ds.finish()
        }
    }
    pub enum GameResponseOffset {}
    #[derive(Copy, Clone, PartialEq)]

    pub struct GameResponse<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for GameResponse<'a> {
        type Inner = GameResponse<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf, loc },
            }
        }
    }

    impl<'a> GameResponse<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            GameResponse { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args GameResponseArgs,
        ) -> flatbuffers::WIPOffset<GameResponse<'bldr>> {
            let mut builder = GameResponseBuilder::new(_fbb);
            if let Some(x) = args.body {
                builder.add_body(x);
            }
            builder.add_body_type(args.body_type);
            builder.finish()
        }

        pub const VT_BODY_TYPE: flatbuffers::VOffsetT = 4;
        pub const VT_BODY: flatbuffers::VOffsetT = 6;

        #[inline]
        pub fn body_type(&self) -> GameResponseBody {
            self._tab
                .get::<GameResponseBody>(GameResponse::VT_BODY_TYPE, Some(GameResponseBody::NONE))
                .unwrap()
        }
        #[inline]
        pub fn body(&self) -> flatbuffers::Table<'a> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Table<'a>>>(
                    GameResponse::VT_BODY,
                    None,
                )
                .unwrap()
        }
        #[inline]
        #[allow(non_snake_case)]
        pub fn body_as_response(&self) -> Option<Response<'a>> {
            if self.body_type() == GameResponseBody::Response {
                let u = self.body();
                Some(Response::init_from_table(u))
            } else {
                None
            }
        }

        #[inline]
        #[allow(non_snake_case)]
        pub fn body_as_state_response(&self) -> Option<StateResponse<'a>> {
            if self.body_type() == GameResponseBody::StateResponse {
                let u = self.body();
                Some(StateResponse::init_from_table(u))
            } else {
                None
            }
        }
    }

    impl flatbuffers::Verifiable for GameResponse<'_> {
        #[inline]
        fn run_verifier(
            v: &mut flatbuffers::Verifier,
            pos: usize,
        ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
            use self::flatbuffers::Verifiable;
            v.visit_table(pos)?
                .visit_union::<GameResponseBody, _>(
                    &"body_type",
                    Self::VT_BODY_TYPE,
                    &"body",
                    Self::VT_BODY,
                    true,
                    |key, v, pos| match key {
                        GameResponseBody::Response => v
                            .verify_union_variant::<flatbuffers::ForwardsUOffset<Response>>(
                                "GameResponseBody::Response",
                                pos,
                            ),
                        GameResponseBody::StateResponse => v
                            .verify_union_variant::<flatbuffers::ForwardsUOffset<StateResponse>>(
                                "GameResponseBody::StateResponse",
                                pos,
                            ),
                        _ => Ok(()),
                    },
                )?
                .finish();
            Ok(())
        }
    }
    pub struct GameResponseArgs {
        pub body_type: GameResponseBody,
        pub body: Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>,
    }
    impl<'a> Default for GameResponseArgs {
        #[inline]
        fn default() -> Self {
            GameResponseArgs {
                body_type: GameResponseBody::NONE,
                body: None, // required field
            }
        }
    }
    pub struct GameResponseBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> GameResponseBuilder<'a, 'b> {
        #[inline]
        pub fn add_body_type(&mut self, body_type: GameResponseBody) {
            self.fbb_.push_slot::<GameResponseBody>(
                GameResponse::VT_BODY_TYPE,
                body_type,
                GameResponseBody::NONE,
            );
        }
        #[inline]
        pub fn add_body(&mut self, body: flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>) {
            self.fbb_
                .push_slot_always::<flatbuffers::WIPOffset<_>>(GameResponse::VT_BODY, body);
        }
        #[inline]
        pub fn new(
            _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        ) -> GameResponseBuilder<'a, 'b> {
            let start = _fbb.start_table();
            GameResponseBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<GameResponse<'a>> {
            let o = self.fbb_.end_table(self.start_);
            self.fbb_.required(o, GameResponse::VT_BODY, "body");
            flatbuffers::WIPOffset::new(o.value())
        }
    }

    impl std::fmt::Debug for GameResponse<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut ds = f.debug_struct("GameResponse");
            ds.field("body_type", &self.body_type());
            match self.body_type() {
                GameResponseBody::Response => {
                    if let Some(x) = self.body_as_response() {
                        ds.field("body", &x)
                    } else {
                        ds.field(
                            "body",
                            &"InvalidFlatbuffer: Union discriminant does not match value.",
                        )
                    }
                }
                GameResponseBody::StateResponse => {
                    if let Some(x) = self.body_as_state_response() {
                        ds.field("body", &x)
                    } else {
                        ds.field(
                            "body",
                            &"InvalidFlatbuffer: Union discriminant does not match value.",
                        )
                    }
                }
                _ => {
                    let x: Option<()> = None;
                    ds.field("body", &x)
                }
            };
            ds.finish()
        }
    }
    #[inline]
    #[deprecated(since = "2.0.0", note = "Deprecated in favor of `root_as...` methods.")]
    pub fn get_root_as_game_response<'a>(buf: &'a [u8]) -> GameResponse<'a> {
        unsafe { flatbuffers::root_unchecked::<GameResponse<'a>>(buf) }
    }

    #[inline]
    #[deprecated(since = "2.0.0", note = "Deprecated in favor of `root_as...` methods.")]
    pub fn get_size_prefixed_root_as_game_response<'a>(buf: &'a [u8]) -> GameResponse<'a> {
        unsafe { flatbuffers::size_prefixed_root_unchecked::<GameResponse<'a>>(buf) }
    }

    #[inline]
    /// Verifies that a buffer of bytes contains a `GameResponse`
    /// and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_game_response_unchecked`.
    pub fn root_as_game_response(
        buf: &[u8],
    ) -> Result<GameResponse, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::root::<GameResponse>(buf)
    }
    #[inline]
    /// Verifies that a buffer of bytes contains a size prefixed
    /// `GameResponse` and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `size_prefixed_root_as_game_response_unchecked`.
    pub fn size_prefixed_root_as_game_response(
        buf: &[u8],
    ) -> Result<GameResponse, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::size_prefixed_root::<GameResponse>(buf)
    }
    #[inline]
    /// Verifies, with the given options, that a buffer of bytes
    /// contains a `GameResponse` and returns it.
    /// Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_game_response_unchecked`.
    pub fn root_as_game_response_with_opts<'b, 'o>(
        opts: &'o flatbuffers::VerifierOptions,
        buf: &'b [u8],
    ) -> Result<GameResponse<'b>, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::root_with_opts::<GameResponse<'b>>(opts, buf)
    }
    #[inline]
    /// Verifies, with the given verifier options, that a buffer of
    /// bytes contains a size prefixed `GameResponse` and returns
    /// it. Note that verification is still experimental and may not
    /// catch every error, or be maximally performant. For the
    /// previous, unchecked, behavior use
    /// `root_as_game_response_unchecked`.
    pub fn size_prefixed_root_as_game_response_with_opts<'b, 'o>(
        opts: &'o flatbuffers::VerifierOptions,
        buf: &'b [u8],
    ) -> Result<GameResponse<'b>, flatbuffers::InvalidFlatbuffer> {
        flatbuffers::size_prefixed_root_with_opts::<GameResponse<'b>>(opts, buf)
    }
    #[inline]
    /// Assumes, without verification, that a buffer of bytes contains a GameResponse and returns it.
    /// # Safety
    /// Callers must trust the given bytes do indeed contain a valid `GameResponse`.
    pub unsafe fn root_as_game_response_unchecked(buf: &[u8]) -> GameResponse {
        flatbuffers::root_unchecked::<GameResponse>(buf)
    }
    #[inline]
    /// Assumes, without verification, that a buffer of bytes contains a size prefixed GameResponse and returns it.
    /// # Safety
    /// Callers must trust the given bytes do indeed contain a valid size prefixed `GameResponse`.
    pub unsafe fn size_prefixed_root_as_game_response_unchecked(buf: &[u8]) -> GameResponse {
        flatbuffers::size_prefixed_root_unchecked::<GameResponse>(buf)
    }
    #[inline]
    pub fn finish_game_response_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<GameResponse<'a>>,
    ) {
        fbb.finish(root, None);
    }

    #[inline]
    pub fn finish_size_prefixed_game_response_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<GameResponse<'a>>,
    ) {
        fbb.finish_size_prefixed(root, None);
    }
} // pub mod MCFS
