use crate::huffman;
use crate::parsing::{Class, Segments, ComponentId};

#[derive(Default, Debug)]
struct PrevDC {
    lum: i16,
    cb: i16,
    cr: i16,
}

pub fn scan_blocks(segments: &mut Segments) -> Vec<[i16; 64]> {
    let mut blocks: Vec<[i16; 64]> = Vec::new();
    let mut block:[i16; 64] = [0; 64];

    let bytes = segments.start_of_scan.as_ref().unwrap().data;

    let mut bit_stream = huffman::BitStream::new(bytes);
    let mut table = &mut segments.huffman_tables[0]; // DC, Lum

    let mut curr_component = ComponentId::LumY;
    let mut n_data_units = 4;
    let mut i = 0;
    let mut prev_dc = PrevDC::default();

    while let Some(bit) = bit_stream.next_bit() {
        // print!("{bit:01b}");

        if let Ok(Some(category)) = table.tree.forward(bit) {
            // println!(" ");
            // println!("i: {}, c: {:x}, {:?}, {:?}, pos: {}", i, category, &curr_component, &table.class, bit_stream.get_pos());
            
            if table.class == Class::DC {
                // dbg!(&prev_dc);
                match curr_component {
                    ComponentId::LumY => {
                        // println!("New AC Lum");
                        table = &mut segments.huffman_tables[1]; // AC, Lum

                        block[i] = bit_stream.get_coeff(category) + prev_dc.lum;
                        prev_dc.lum = block[i];
                        assert_eq!(i, 0);
                        i += 1;
                        continue;
                    },
                    ComponentId::ChromCb => {
                        table = &mut segments.huffman_tables[3]; // AC, Chr

                        block[i] = bit_stream.get_coeff(category) + prev_dc.cb;
                        prev_dc.cb = block[i];
                        assert_eq!(i, 0);
                        i += 1;
                        continue;
                    }
                    ComponentId::ChromCr => {
                        table = &mut segments.huffman_tables[3]; // AC, Chr

                        block[i] = bit_stream.get_coeff(category) + prev_dc.cr;
                        prev_dc.cr = block[i];
                        assert_eq!(i, 0);
                        i += 1;
                        continue;
                    }
                }
            } else {       // Class::AC
                if category == 0 { // End of Block
                    match curr_component {
                        ComponentId::LumY => {
                            if n_data_units > 1 {
                                // println!("new DC Lum");
                                table = &mut segments.huffman_tables[0]; // DC, Lum
                                n_data_units -= 1;
                            } else {
                                table = &mut segments.huffman_tables[2]; // DC, Chr
                                curr_component = ComponentId::ChromCb;
                                n_data_units = 4;
                            }
                        },
                        ComponentId::ChromCb => {
                            table = &mut segments.huffman_tables[2]; // DC, Chr
                            curr_component = ComponentId::ChromCr;
                        }
                        ComponentId::ChromCr => {
                            table = &mut segments.huffman_tables[0]; // DC, Lum
                            curr_component = ComponentId::LumY;
                        }
                    }
                    // println!("EOB: {:?}", block);

                    blocks.push(block);
                    block = [0; 64];
                    i = 0;
                    continue;

                } else if i >= 63 {
                    match curr_component {
                        ComponentId::LumY => {
                            if n_data_units > 1 {
                                // println!("new DC Lum");
                                table = &mut segments.huffman_tables[0]; // DC, Lum
                                n_data_units -= 1;
                            } else {
                                table = &mut segments.huffman_tables[2]; // DC, Chr
                                curr_component = ComponentId::ChromCb;
                                n_data_units = 4;
                            }
                        },
                        ComponentId::ChromCb => {
                            table = &mut segments.huffman_tables[2]; // DC, Chr
                            curr_component = ComponentId::ChromCr;
                        }
                        ComponentId::ChromCr => {
                            table = &mut segments.huffman_tables[0]; // DC, Lum
                            curr_component = ComponentId::LumY;
                        }
                    }

                    block[63] = bit_stream.get_coeff(category & 0x0F);

                    // println!("{:?}", block);
                    blocks.push(block);
                    block = [0; 64];
                    i = 0;
                    continue;

                } else {
                    i += (category as usize) >> 4;
                    block[i] = bit_stream.get_coeff(category & 0x0F);
                    if i >= 63 {
                        match curr_component {
                            ComponentId::LumY => {
                                if n_data_units > 1 {
                                    // println!("new DC Lum");
                                    table = &mut segments.huffman_tables[0]; // DC, Lum
                                    n_data_units -= 1;
                                } else {
                                    table = &mut segments.huffman_tables[2]; // DC, Chr
                                    curr_component = ComponentId::ChromCb;
                                    n_data_units = 4;
                                }
                            },
                            ComponentId::ChromCb => {
                                table = &mut segments.huffman_tables[2]; // DC, Chr
                                curr_component = ComponentId::ChromCr;
                            }
                            ComponentId::ChromCr => {
                                table = &mut segments.huffman_tables[0]; // DC, Lum
                                curr_component = ComponentId::LumY;
                            }
                        }
                        // println!("{:?}", block);
                        blocks.push(block);
                        block = [0; 64];
                        i = 0;
                        continue;

                    } else {
                        i += 1;
                    }
                }
            }
        };
    }

    blocks
}











#[cfg(test)]
mod test {
    use std::fs;
    use crate::parsing::parse;
    use crate::huffman::BitStream;
    // use super::*;

    #[test]
    fn test_huffman_and_bits() {
        let bytes = [0b11110111, 0b11110100];

        let mut bit_stream = BitStream::new(&bytes);
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let tree = &mut parse(&data).huffman_tables[0].tree;
        let mut value = 0;
        for _ in 0..16 {
            let bit = bit_stream.next_bit().unwrap();
            if let Ok(Some(x)) = tree.forward(bit) {
                value = bit_stream.get_coeff(x);
                break
            };
        }
        assert_eq!(value, 127);

        for _ in 0..16 {
            let bit = bit_stream.next_bit().unwrap();
            if let Ok(Some(x)) = tree.forward(bit) {
                value = bit_stream.get_coeff(x);
                break
            };
        }
        assert_eq!(value, -1);
    }
}