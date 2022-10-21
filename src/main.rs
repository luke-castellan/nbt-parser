// TODO: Remove attributes and fix warnings manually
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use std::any::Any;
use std::fs::File;
use std::io::{BufReader, Read, Write};
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
        let nbt_type = read_nbt(std::io::Read::by_ref(&mut reader), None).unwrap();
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
        "".to_string()
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
                        childs.push(Box::new(child_type));
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

fn write_nbt<T>(writer: &mut T, data: &NBTType, specific: Option<u8>) where T: Write {
    match data {
        NBTType::TAG_End => {
            writer.by_ref().write(&(0 as u8).to_be_bytes());
        }
        NBTType::TAG_Byte(field_name, byte) => {
            write_header(writer.by_ref(), &1, field_name);
            write_u8(writer.by_ref(), byte);
        }
        NBTType::TAG_Short(_, _) => {}
        NBTType::TAG_Int(_, _) => {}
        NBTType::TAG_Long(_, _) => {}
        NBTType::TAG_Float(_, _) => {}
        NBTType::TAG_Double(_, _) => {}
        NBTType::TAG_Byte_Array(_, _) => {}
        NBTType::TAG_String(_, _) => {}
        NBTType::TAG_List(_, _) => {}
        NBTType::TAG_Compound(field_name, childs) => {
            write_header(writer.by_ref(), &10, field_name);
            for nbttype in childs.iter() {
                write_nbt(writer.by_ref(), &**nbttype, None);
            }
        }
        NBTType::TAG_Int_Array(_, _) => {}
        NBTType::TAG_Long_Array(_, _) => {}
    }
}

fn write_u8<T>(writer: &mut T, data: &u8) where T: Write {
    writer.by_ref().write(data.to_be_bytes().as_ref()).unwrap();
}

fn write_u16<T>(writer: &mut T, data: &u16) where T: Write {
    writer.by_ref().write(data.to_be_bytes().as_ref()).unwrap();
}

fn write_string<T>(writer: &mut T, data: &String) where T: Write {
    write_u16(writer.by_ref(), &(data.len() as u16));
    writer.by_ref().write(data.as_bytes()).unwrap();
}

fn write_header<T>(writer: &mut T, id: &u8, field_name: &String) where T: Write {
    if field_name.len() > 0 {
        write_u8(writer.by_ref(), id);
        write_string(writer.by_ref(), field_name);
    }
}

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::any::Any;
    use crate::{NBTType, read_nbt, write_nbt};

    #[test]
    fn test_serializer() {
        let mut inner: Vec<Box<NBTType>> = Vec::new();
        inner.push(Box::new(NBTType::TAG_Byte("sample byte".to_string(), 3)));
        inner.push(Box::new(NBTType::TAG_End));
        let mut data = NBTType::TAG_Compound("hello test".to_string(), inner);

        let mut bytes: Vec<u8> = Vec::new();
        write_nbt(&mut bytes, &data, None);

        let mut deserialized_data = read_nbt(&mut bytes.as_slice(), None).unwrap();
        assert_eq!(data, deserialized_data);
    }
}