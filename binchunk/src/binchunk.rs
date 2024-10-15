/// fixed signature number for luac file
const LUA_SIGNATURE: [u8; 4] = u32::to_be_bytes(0x1B4C7561);
/// version of lua-rs is 5.3.6, computation is a.b.c to (a * 16 + b)
const LUAC_VERSION: u8 = 0x53;
/// format number is default to be zero, I set it to 0x4A
const LUAC_FORMAT: u8 = 0;
/// fixed luac_data number for luac file
const LUAC_DATA: [u8; 6] = [0x19, 0x93, 0x0D, 0x0A, 0x1A, 0x0A];
/// set cint_size for lua
const CINT_SIZE: u8 = 0x04;
/// set sizet_size for lua
const CSIZET_SIZE: u8 = 0x08;
/// set instruction_size for lua
const INSTRUCTION_SIZE: u8 = 0x04;
/// set lua_integer_size for lua
const LUA_INTEGER_SIZE: u8 = 0x08;
/// set lua_number_size for lua
const LUA_NUMBER_SIZE: u8 = 0x08;
/// set luac_int to a fixed number 0x5678 to determine whether the target is be or le
const LUAC_INT: i64 = 0x5678;
/// set luac_num to a fixed number 370.5 to see its float point number's representation
const LUAC_NUM: f64 = 370.5;

#[derive(Clone, Debug, Default)]
pub struct BinaryChunk {
    header: Header,
    size_of_upvalues: u8,
    main_fn: Box<Prototype>,
}

#[derive(Clone, Debug, Default)]
pub struct Upvalue {
    pub in_stack: u8,
    pub idx: u8,
}

#[derive(Clone, Debug, Default)]
pub enum Constant {
    #[default]
    Nil, // 0x00
    Boolean(bool),       // 0x01
    Integer(i64),        // 0x03
    Number(f64),         // 0x13
    ShortString(String), // 0x04
    LongString(String),  // 0x14
}

#[derive(Clone, Debug, Default)]
struct Header {
    signature: [u8; 4],
    version: u8,
    format: u8,
    luac_data: [u8; 6],
    cint_size: u8,
    sizet_size: u8,
    instruction_size: u8,
    lua_integer_size: u8,
    lua_number_size: u8,
    luac_int: i64,
    luac_num: f64,
}

#[derive(Clone, Debug, Default)]
pub struct LocVar {
    pub var_name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Prototype {
    pub source: String,
    pub line_defined: u32,
    pub last_line_defined: u32,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub upvalues: Vec<Upvalue>,
    pub protos: Vec<Prototype>,
    pub line_info: Vec<u32>,
    pub loc_vars: Vec<LocVar>,
    pub upvalue_names: Vec<String>,
}

pub fn un_dump(data: &[u8]) -> Option<Prototype> {
    let mut reader = Reader::new(data);
    reader.check_head();
    reader.read_byte();
    // println!("{:?}", reader.read_string());
    reader.read_proto("")
}

#[derive(Clone, Debug, Default)]
struct Reader {
    data: Vec<u8>,
    index: usize,
}

impl Reader {
    fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned(),
            index: 0,
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        self.index += 1;
        if self.data.len() < self.index {
            None
        } else {
            Some(self.data[self.index - 1])
        }
    }

    fn read_u32(&mut self) -> Option<u32> {
        self.index += 4;
        if self.data.len() < self.index {
            None
        } else {
            Some(u32::from_ne_bytes(
                self.data[self.index - 4..self.index].try_into().ok()?,
            ))
        }
    }

    fn read_u64(&mut self) -> Option<u64> {
        self.index += 8;
        if self.data.len() < self.index {
            None
        } else {
            Some(u64::from_ne_bytes(
                self.data[self.index - 8..self.index].try_into().ok()?,
            ))
        }
    }

    fn read_lua_integer(&mut self) -> Option<i64> {
        self.index += 8;
        if self.data.len() < self.index {
            None
        } else {
            Some(i64::from_ne_bytes(
                self.data[self.index - 8..self.index].try_into().ok()?,
            ))
        }
    }

    fn read_lua_number(&mut self) -> Option<f64> {
        self.index += 8;
        if self.data.len() < self.index {
            None
        } else {
            Some(f64::from_ne_bytes(
                self.data[self.index - 8..self.index].try_into().ok()?,
            ))
        }
    }

    fn read_string(&mut self) -> Option<String> {
        let mut size = self.read_byte()? as u64;
        if size == 0 {
            return Some(String::new());
        }
        if size == 0xFF {
            size = self.read_u64()?;
        }
        String::from_utf8(self.read_bytes(size as usize - 1)?).ok()
    }

    fn read_bytes(&mut self, n: usize) -> Option<Vec<u8>> {
        self.index += n;
        if self.data.len() < self.index {
            None
        } else {
            Some(self.data[self.index - n..self.index].to_owned())
        }
    }

    fn check_head(&mut self) {
        if self
            .read_bytes(4)
            .expect("Not enough bytes for a header. [signature]")[..4]
            .ne(&LUA_SIGNATURE)
        {
            panic!("not a precompiled chunk!");
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [version]")
            .ne(&LUAC_VERSION)
        {
            panic!("version mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [format]")
            .ne(&LUAC_FORMAT)
        {
            panic!("format mismatch!")
        }
        if self
            .read_bytes(6)
            .expect("Not enough bytes for a header. [luac_data]")[..6]
            .ne(&LUAC_DATA)
        {
            panic!("version mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [cint_size]")
            .ne(&CINT_SIZE)
        {
            panic!("int size mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [c_size_t_size]")
            .ne(&CSIZET_SIZE)
        {
            panic!("size_t size mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [instruction_size]")
            .ne(&INSTRUCTION_SIZE)
        {
            panic!("instruction size mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [lua_integer_size]")
            .ne(&LUA_INTEGER_SIZE)
        {
            panic!("lua_Integer size mismatch!")
        }
        if self
            .read_byte()
            .expect("Not enough bytes for a header. [lua_number_size]")
            .ne(&LUA_NUMBER_SIZE)
        {
            panic!("lua_Number size mismatch!")
        }

        if self
            .read_lua_integer()
            .expect("Not enough bytes for a header. [lua_int]")
            .ne(&LUAC_INT)
        {
            panic!("endianness mismatch!")
        }
        if self
            .read_lua_number()
            .expect("Not enough bytes for a header. [lua_num]")
            .ne(&LUAC_NUM)
        {
            panic!("float format mismatch!")
        }
    }

    fn read_proto(&mut self, parent_source: &str) -> Option<Prototype> {
        let mut source = self.read_string()?;
        if source.is_empty() {
            source = parent_source.to_owned();
        }
        Some(Prototype {
            source: source.clone(),
            line_defined: self.read_u32()?,
            last_line_defined: self.read_u32()?,
            num_params: self.read_byte()?,
            is_vararg: self.read_byte()?,
            max_stack_size: self.read_byte()?,
            code: self.read_code()?,
            constants: self.read_constants()?,
            upvalues: self.read_upvalues()?,
            protos: self.read_protos(&source)?,
            line_info: self.read_line_info()?,
            loc_vars: self.read_loc_vars()?,
            upvalue_names: self.read_upvalue_names()?,
        })
    }

    fn read_code(&mut self) -> Option<Vec<u32>> {
        let mut code = vec![0u32; self.read_u32()? as usize];
        for num in code.iter_mut() {
            *num = self.read_u32()?;
        }
        Some(code)
    }

    fn read_constants(&mut self) -> Option<Vec<Constant>> {
        let mut constants = vec![Default::default(); self.read_u32()? as usize];
        for constant in constants.iter_mut() {
            *constant = self.read_constant()?;
        }

        Some(constants)
    }

    fn read_constant(&mut self) -> Option<Constant> {
        Some(match self.read_byte()? {
            0x00 => Constant::Nil,
            0x01 => Constant::Boolean(self.read_byte()? != 0),
            0x03 => Constant::Integer(self.read_lua_integer()?),
            0x13 => Constant::Number(self.read_lua_number()?),
            0x04 => Constant::ShortString(self.read_string()?),
            0x14 => Constant::LongString(self.read_string()?),
            _ => panic!("corrupted!"),
        })
    }

    fn read_upvalues(&mut self) -> Option<Vec<Upvalue>> {
        let mut upvalues = vec![Default::default(); self.read_u32()? as usize];
        for upvalue in upvalues.iter_mut() {
            *upvalue = Upvalue {
                in_stack: self.read_byte()?,
                idx: self.read_byte()?,
            };
        }
        Some(upvalues)
    }

    fn read_protos(&mut self, parent_source: &str) -> Option<Vec<Prototype>> {
        let mut protos = vec![Default::default(); self.read_u32()? as usize];
        for proto in protos.iter_mut() {
            *proto = self.read_proto(parent_source)?;
        }
        Some(protos)
    }

    fn read_line_info(&mut self) -> Option<Vec<u32>> {
        let mut line_info = vec![Default::default(); self.read_u32()? as usize];
        for line in line_info.iter_mut() {
            *line = self.read_u32()?;
        }
        Some(line_info)
    }

    fn read_loc_vars(&mut self) -> Option<Vec<LocVar>> {
        let mut loc_vars = vec![Default::default(); self.read_u32()? as usize];
        for loc_var in loc_vars.iter_mut() {
            *loc_var = LocVar {
                var_name: self.read_string()?,
                start_pc: self.read_u32()?,
                end_pc: self.read_u32()?,
            };
        }
        Some(loc_vars)
    }

    fn read_upvalue_names(&mut self) -> Option<Vec<String>> {
        let mut names = vec![Default::default(); self.read_u32()? as usize];
        for name in names.iter_mut() {
            *name = self.read_string()?;
        }
        Some(names)
    }
}
