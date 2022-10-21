use std::fs::File;
use std::io::{BufReader, Read};
use std::process::exit;
use clap::Parser;
use flate2::read::GzDecoder;

#[derive(Parser)]
struct Args {
    file_name: String,
    #[arg(short, long)]
    gzip: bool,
}

fn main() {
    let cli_args: Args = Args::parse();
    let file_name = cli_args.file_name;

    let file = File::open(file_name).unwrap();

    if cli_args.gzip {
        let mut reader = GzDecoder::new(file);
        let nbt_type = read_nbt(reader.by_ref(), None).unwrap();
        println!("{:#?}", nbt_type);
    } else {
        let mut reader = BufReader::new(file);
        let nbt_type = read_nbt(reader.by_ref(), None).unwrap();
        println!("{:#?}", nbt_type);
    }
}

fn read_nbt<T>(reader: &mut T, specific: Option<u8>) -> Result<NBTType, ()> where T: Read {
    let type_number = match specific {
        None => {
            let mut buf: [u8; 1] = [0; 1];
            reader.by_ref().take(1).read(&mut buf);
            u8::from_be_bytes(buf)
        }
        Some(value) => {
            value
        }
    };
    let field_name = if specific.is_none() && type_number != 0 {
        read_string(reader.by_ref()).unwrap()
    } else {
        "None".to_string()
    };

    match type_number {
        // End
        0 => {
            Ok(NBTType::TAG_End)
        },

        // Byte
        1 => {
            let final_value = read_u8(reader.by_ref()).unwrap();
            Ok(NBTType::TAG_Byte(field_name, final_value))
        },

        // Short
        2 => {
            let mut buf: [u8; 2] = [0; 2];
            reader.by_ref().take(2).read(&mut buf);
            let final_value = i16::from_be_bytes(buf);
            Ok(NBTType::TAG_Short(field_name, final_value))
        },

        // Int
        3 => {
            let mut buf: [u8; 4] = [0; 4];
            reader.by_ref().take(4).read(&mut buf);
            let final_value = i32::from_be_bytes(buf);
            Ok(NBTType::TAG_Int(field_name, final_value))
        },

        // Long
        4 => {
            let mut buf: [u8; 8] = [0; 8];
            reader.by_ref().take(8).read(&mut buf);
            let final_value = i64::from_be_bytes(buf);
            Ok(NBTType::TAG_Long(field_name, final_value))
        },

        // Float
        5 => {
            let mut buf: [u8; 4] = [0; 4];
            reader.by_ref().take(4).read(&mut buf);
            let final_value = f32::from_be_bytes(buf);
            Ok(NBTType::TAG_Float(field_name, final_value))
        },

        // Double
        6 => {
            let mut buf: [u8; 8] = [0; 8];
            reader.by_ref().take(8).read(&mut buf);
            let final_value = f64::from_be_bytes(buf);
            Ok(NBTType::TAG_Double(field_name, final_value))
        },

        // Byte Array
        7 => {
            let array_length = read_i32(reader.by_ref()).unwrap();
            let mut array: Vec<u8> = Vec::new();
            reader.by_ref().take(array_length as u64).read_to_end(&mut array);
            Ok(NBTType::TAG_Byte_Array(field_name, array))
        },

        // String
        8 => {
            let string_content = read_string(reader.by_ref()).unwrap();
            Ok(NBTType::TAG_String(field_name, string_content))
        },

        // List
        9 => {
            let type_id = read_u8(reader.by_ref()).unwrap();
            let list_length = read_i32(reader.by_ref()).unwrap();
            let mut content: Vec<Box<NBTType>> = Vec::new();

            for _ in 0..list_length {
                content.push(Box::new(read_nbt(reader.by_ref(), Some(type_id)).unwrap()));
            }
            Ok(NBTType::TAG_List(field_name, content))
        },

        // Compound
        10 => {
            let mut childs: Vec<Box<NBTType>> = Vec::new();

            loop {
                let child_type = read_nbt(reader.by_ref(), None).unwrap();
                match child_type {
                    NBTType::TAG_End => {
                        break;
                    },
                    _ => {
                        childs.push(Box::new(child_type));
                    }
                }
            }
            Ok(NBTType::TAG_Compound(field_name, childs))
        },

        // Int Array
        11 => {
            let array_length = read_i32(reader.by_ref()).unwrap();
            let mut array: Vec<i32> = Vec::new();
            for _ in 0..array_length {
                array.push(read_i32(reader.by_ref()).unwrap());
            }
            Ok(NBTType::TAG_Int_Array(field_name, array))
        },

        // Long Array
        12 => {
            let array_length = read_i32(reader.by_ref()).unwrap();
            let mut array: Vec<i64> = Vec::new();
            for _ in 0..array_length {
                array.push(read_i64(reader.by_ref()).unwrap());
            }
            Ok(NBTType::TAG_Long_Array(field_name, array))
        },

        _ => {
            eprintln!("Error: Read invalid NBT Type ID");
            exit(1);
        }
    }
}

fn read_string<T>(mut reader: T) -> Result<String, ()>
where T: Read
{
    let mut buf: [u8; 2] = [0; 2];
    let bytes_read = reader.by_ref().take(2).read(&mut buf).unwrap();
    if bytes_read < 2 {
        return Err(())
    }
    let string_length = u16::from_be_bytes(buf);
    let mut string_buf: Vec<u8> = Vec::new();
    reader.take(string_length as u64).read_to_end(&mut string_buf).unwrap();

    Ok(String::from_utf8(string_buf).expect("Error parsing UTF"))
}

fn read_u8<T>(mut reader: T) -> Result<u8, ()>
where T: Read
{
    let mut buf: [u8; 1] = [0; 1];
    reader.by_ref().take(1).read(&mut buf);
    Ok(u8::from_be_bytes(buf))
}

fn read_i32<T>(mut reader: T) -> Result<i32, ()>
where T: Read
{
    let mut buf: [u8; 4] = [0; 4];
    reader.by_ref().take(4).read(&mut buf);
    Ok(i32::from_be_bytes(buf))
}

fn read_i64<T>(mut reader: T) -> Result<i64, ()>
where T: Read
{
    let mut buf: [u8; 8] = [0; 8];
    reader.by_ref().take(8).read(&mut buf);
    Ok(i64::from_be_bytes(buf))
}

#[derive(Debug)]
enum NBTType {
    TAG_End,
    TAG_Byte(String, u8),
    TAG_Short(String, i16),
    TAG_Int(String, i32),
    TAG_Long(String, i64),
    TAG_Float(String, f32),
    TAG_Double(String, f64),
    TAG_Byte_Array(String, Vec<u8>),
    TAG_String(String, String),
    TAG_List(String, Vec<Box<NBTType>>),
    TAG_Compound(String, Vec<Box<NBTType>>),
    TAG_Int_Array(String, Vec<i32>),
    TAG_Long_Array(String, Vec<i64>)
}
