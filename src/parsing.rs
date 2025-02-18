use crate::huffman;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Segments<'a> {
    pub application: Option<Application0<'a>>,
    pub quantization_tables: Vec<QuantizationTable>,
    pub start_of_frame: Option<StartOfFrame>,
    pub huffman_tables: Vec<HuffmanTable>,
    pub start_of_scan: Option<StartOfScan<'a>>,
    pub comments: Vec<String>,
    pub scan: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum DensityUnit {
    NoUnit,
    PixelsPerInch,
    PixelsPerCm,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Application0<'a> {
    pub identifier: String,
    pub version: (u8, u8),
    pub density_unit: DensityUnit,
    pub density: (u16, u16),
    pub thumbnail_size: (u8, u8),
    pub thumbnail: &'a [u8],
}

impl<'a> Application0<'a> {
    fn new(bytes: &'a [u8]) -> Application0<'a> {
        Application0 {
            identifier: String::from_utf8(bytes[0..5].to_vec()).expect("Identifier parsing"),
            version: (bytes[5], bytes[6]),
            density_unit: match bytes[7] {
                0 => DensityUnit::NoUnit,
                1 => DensityUnit::PixelsPerInch,
                2 => DensityUnit::PixelsPerCm,
                _ => panic!("Wrong density unit"),
            },
            density: (
                u16::from_be_bytes([bytes[8], bytes[9]]),
                u16::from_be_bytes([bytes[10], bytes[11]]),
            ),
            thumbnail_size: (bytes[12], bytes[13]),
            thumbnail: if bytes[12] != 0 && bytes[13] != 0 {
                &bytes[14..]
            } else {
                &[]
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Destination {
    Luminance,
    Chrominance,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct QuantizationTable {
    pub destination: Destination,
    pub table: [u8; 64],
}

impl QuantizationTable {
    fn new(bytes: &[u8]) -> QuantizationTable {
        if bytes.len() < 65 {
            panic!("Not enough bytes for a quantization table");
        }

        QuantizationTable {
            destination: match bytes[0] & 0x0F {
                0 => Destination::Luminance,
                1 => Destination::Chrominance,
                _ => panic!("Wrong destination"),
            },
            table: bytes[1..=64].try_into().expect("Slice should be exactly 64 bytes"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ComponentId {
    LumY,
    ChromCb,
    ChromCr,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ComponentSOF {
    pub id: ComponentId,
    pub factors: (u8, u8),
    pub quantization_table: u8,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct StartOfFrame {
    pub precision: u8,
    pub height: u16,
    pub width: u16,
    pub components: Vec<ComponentSOF>,
}

impl StartOfFrame {
    fn new(bytes: &[u8]) -> StartOfFrame {
        StartOfFrame {
            precision: bytes[0],
            height: u16::from_be_bytes([bytes[1], bytes[2]]),
            width: u16::from_be_bytes([bytes[3], bytes[4]]),
            components: (0..bytes[5] as usize)
                .map(|i| ComponentSOF {
                    id: match bytes[6 + i * 3] {
                        1 => ComponentId::LumY,
                        2 => ComponentId::ChromCb,
                        3 => ComponentId::ChromCr,
                        _ => panic!("Wrong component id"),
                    },
                    factors: (
                        bytes[7 + i * 3] >> 4,
                        bytes[7 + i * 3] & 0x0F,
                    ),
                    quantization_table: bytes[8 + i * 3]
                }).collect()
        }
    }
    
}

#[derive(Debug, PartialEq)]
pub enum Class {
    AC,
    DC,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct HuffmanTable {
    pub class: Class,
    pub destination: Destination,
    pub tree: huffman::Tree,
}

impl HuffmanTable {
    fn new(bytes: &[u8]) -> HuffmanTable {
        HuffmanTable {
            class: match bytes[0] >> 4 {
                0 => Class::DC,
                1 => Class::AC,
                _ => panic!("Wrong class"),
            },
            destination: match bytes[0] & 0x0F {
                0 => Destination::Luminance,
                1 => Destination::Chrominance,
                _ => panic!("Wrong class"),
            },
            tree: {
                let mut tree = huffman::Tree::new();
                let mut pos: usize = 0;
                let mut code: u16 = 0;
                for (n, &q) in bytes[1..17].iter().enumerate() {
                    for _ in 0..q {
                        let symbol = bytes[17+pos];
                        tree.insert_code(code, (n+1) as u8, symbol);
                        pos += 1;
                        code += 1;
                    }
                    code <<= 1;
                }
                tree
            }
        }
    } 
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ComponentSOS {
    dc_table: Destination,
    ac_table: Destination,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct StartOfScan<'a>{
    pub components: Vec<ComponentSOS>,
    pub data: &'a [u8],
}

impl<'a> StartOfScan<'a> {
    fn new(bytes: &[u8]) -> StartOfScan<'a> {
        StartOfScan {
            components: {
                let mut comps = Vec::new();
                let n_comp = bytes[0] as usize;
                for i in 1..=n_comp {
                    comps.push(ComponentSOS {
                        dc_table: match bytes[i*2] >> 4 {
                            0 => Destination::Luminance,
                            1 => Destination::Chrominance,
                            _ => panic!("Wrong destination"),
                        },
                        ac_table: match bytes[i*2] & 0x0F {
                            0 => Destination::Luminance,
                            1 => Destination::Chrominance,
                            _ => panic!("Wrong destination"),
                        },
                    });
                }
                comps
            },
            data: &[],
        }
    }
}

fn get_lenght(bytes: &[u8]) -> usize {
    u16::from_be_bytes([bytes[0], bytes[1]]) as usize
}

pub fn parse(bytes: &[u8]) -> Segments {
    let mut segments = Segments::default();

    let mut i = 0;
    let mut i_sos = 0;
    while i < bytes.len() {
        if bytes[i] == 0xFF {
            if bytes[i + 1] == 0xE0 {
                let length = get_lenght(&bytes[i+2..=i+3]);
                segments.application = Some(Application0::new(&bytes[i+4..i+2+length]));
                i += length + 1;
            } else if bytes[i + 1] == 0xDB {
                let length = get_lenght(&bytes[i+2..=i+3]);
                segments.quantization_tables.push(QuantizationTable::new(&bytes[i+4..i+2+length]));
                i += length + 1;
            } else if bytes[i + 1] == 0xC0 {
                let length = get_lenght(&bytes[i+2..=i+3]);
                segments.start_of_frame = Some(StartOfFrame::new(&bytes[i+4..i+2+length]));
                i += length + 1;
            } else if bytes[i + 1] == 0xC4 {
                let length = get_lenght(&bytes[i+2..=i+3]);
                segments.huffman_tables.push(HuffmanTable::new(&bytes[i+4..i+2+length]));
                i += length + 1;
            } else if bytes[i + 1] == 0xDA {
                let length = get_lenght(&bytes[i+2..=i+3]);
                segments.start_of_scan = Some(StartOfScan::new(&bytes[i+4..i+2+length]));
                i += length + 1;
                i_sos = i + 1;
            } else if bytes[i + 1] == 0xD9 {
                segments.start_of_scan.as_mut().unwrap().data = &bytes[i_sos..i];
            }
        }
        i += 1;
    }
    segments
}




#[cfg(test)]
mod tests {
    use std::fs;
    use crate::parsing::*;

    #[test]
    fn test_parse_application_segment() {
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let segments = parse(&data);

        assert!(segments.application.is_some(), "Application segment should exist");

        let app0 = segments.application.unwrap();
        assert_eq!(app0.identifier, "JFIF\0", "JFIF identifier mismatch");
        assert_eq!(app0.version, (1, 1), "JFIF version mismatch");
        assert_eq!(app0.density_unit, DensityUnit::PixelsPerInch, "Density unit mismatch");
        assert_eq!(app0.density, (168, 168), "Density values mismatch");
    }

    #[test]
    fn test_parse_quantization_table() {
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let segments = parse(&data);

        assert_eq!(segments.quantization_tables.len(), 2, "Expected two quantization tables");

        let q_table_1 = &segments.quantization_tables[0];
        let q_table_2 = &segments.quantization_tables[1];

        // Check the first few bytes of the first quantization table
        assert_eq!(q_table_1.table[0..5], [2, 1, 1, 2, 1], "Mismatch in first quantization table");
        assert_eq!(q_table_1.table.len(), 64, "Mismatch in len first quantization table");

        // Check the first few bytes of the second quantization table
        assert_eq!(q_table_2.table[0..5], [2, 2, 2, 3, 3], "Mismatch in second quantization table");
        assert_eq!(q_table_2.table.len(), 64, "Mismatch in len second quantization table");
    }

    #[test]
    fn test_parse_start_of_frame() {
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let segments = parse(&data);

        let start_of_frame = &segments.start_of_frame.unwrap();

        assert_eq!(start_of_frame.precision, 8, "Start of frame: precision");
        assert_eq!(start_of_frame.height, 8, "Start of frame: height");
        assert_eq!(start_of_frame.width, 8, "Start of frame: width");

        assert_eq!(start_of_frame.components[0].id, ComponentId::LumY, "Start of frame: width");
        assert_eq!(start_of_frame.components[0].factors, (2,2), "Start of frame: width");
        assert_eq!(start_of_frame.components[0].quantization_table, 0, "Start of frame: width");
        
        assert_eq!(start_of_frame.components[1].id, ComponentId::ChromCb, "Start of frame: width");
        assert_eq!(start_of_frame.components[1].factors, (1,1), "Start of frame: width");
        assert_eq!(start_of_frame.components[1].quantization_table, 1, "Start of frame: width");
        
        assert_eq!(start_of_frame.components[2].id, ComponentId::ChromCr, "Start of frame: width");
        assert_eq!(start_of_frame.components[2].factors, (1,1), "Start of frame: width");
        assert_eq!(start_of_frame.components[2].quantization_table, 1, "Start of frame: width");
    }

    #[test]
    fn test_parse_huffman_table() {
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let segments = parse(&data);

        assert_eq!(segments.huffman_tables.len(), 4, "Expected four Huffman tables");

        assert_eq!(segments.huffman_tables[0].class, Class::DC, "Coeff class DC");
        assert_eq!(segments.huffman_tables[0].destination, Destination::Luminance, "Destination Luminance");
        // assert_eq!(segments.huffman_tables[0].tree, (2, 0, 0), "Symbol: 0, code: 00");
        // assert_eq!(segments.huffman_tables[0].data[1], (3, 1, 2), "Symbol: 1, code: 010");
        // assert_eq!(segments.huffman_tables[0].data[11], (9, 11, 510), "Symbol: 11, code: 111111110");
        assert_eq!(segments.huffman_tables.len(), 4, "Expected four Huffman tables");

    }
}


