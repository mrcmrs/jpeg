use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node {
    symbol: Option<u8>,
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            symbol: None,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    root: Rc<RefCell<Node>>,
    curr_node: Rc<RefCell<Node>>,
}

impl Tree {
    pub fn new() -> Tree {
        let root = Rc::new(RefCell::new(Node::new()));
        Tree {
            root: root.clone(),
            curr_node: root.clone(),
        }
    }

    pub fn insert_code(&mut self, code: u16, length: u8, symbol: u8) {
        // println!("c: {:b}, l: {}, s: {:x}", code, length, symbol);
        let mut node = self.root.clone();
        // Iterate from the most-significant bit (of the given length) down to 0.
        for i in (0..length).rev() {
            let bit = (code >> i) & 1;
            if bit == 0 {
                if node.borrow().left.is_none() {
                    node.borrow_mut().left = Some(Rc::new(RefCell::new(Node::new())));
                }
                let next_node = node.borrow().left.as_ref().unwrap().clone();
                node = next_node;
            } else {
                if node.borrow().right.is_none() {
                    node.borrow_mut().right = Some(Rc::new(RefCell::new(Node::new())));
                }
                let next_node = node.borrow().right.as_ref().unwrap().clone();
                node = next_node;
            }
        }
        // At the leaf, store the symbol.
        node.borrow_mut().symbol = Some(symbol);
    }

    pub fn forward(&mut self, bit: u8) -> Result<Option<u8>, &'static str>{
        // Use a match to select the appropriate child.
        let next = match bit {
            0 => self.curr_node.borrow().left.clone(),
            1 => self.curr_node.borrow().right.clone(),
            _ => return Err("Must be 0 or 1"),
        };
    
        // If a child exists, update curr_node; otherwise, return None.
        if let Some(child) = next {
            if let Some(value) = child.borrow().symbol {  
                self.curr_node = self.root.clone();
                Ok(Some(value))
            } else {
                self.curr_node = child.clone();
                Ok(None)
            }
        } else {
            Err("No child")
        }
    }
    
}


#[warn(dead_code)]
pub struct BitStream<'a> {
    data: &'a [u8],
    curr_byte: usize,
    curr_bit: u8,
    pos: usize,
}

impl<'a> BitStream<'a> {
    pub fn new(bytes: &'a [u8]) -> BitStream<'a> {
        BitStream {
            data: bytes,
            curr_byte: 0,
            curr_bit: 8,
            pos: 0,
        }
    }

    pub fn next_bit(&mut self) -> Option<u8> {
        if self.pos >= self.data.len()*8-1{
            println!("End of scan");
            return None;
        }

        let i = self.curr_byte;
        if self.data[i] == 0xFF && self.data[i+1] != 0x00 {
            println!("Marker detected");
            return None;
        }
        if  self.data[i] == 0x00 && self.data[i-1] == 0xFF {
            println!(" ");
            println!("0x00 detected after 0xFF");
            self.curr_byte += 1;
            self.curr_bit = 8;
        }
        let byte = self.data[self.curr_byte];

        self.curr_bit -= 1;
        let res = (byte >> self.curr_bit) & 1;

        if self.curr_bit == 0 {
            self.curr_bit = 8;
            self.curr_byte += 1;
        }

        self.pos = 8 * self.curr_byte + 7 - self.curr_bit as usize;
        // print!("{res:01b}");

        Some(res)
    }

    pub fn get_coeff(&mut self, category: u8) -> i16 {
        // println!(" ");
        if category == 0 {
            return 0;
        }
        let mut value = 0;
        for _ in 0..category {
            value <<= 1;
            value += self.next_bit().expect("Can't reach the next bit") as i16;
        }
        let vt = 1 << (category - 1);
        if value < vt {
            let vt = (-1) << category;
            value += vt + 1;
        }
        // println!(" ");

        value
    }

    pub fn get_pos(&self) -> usize {
        // return position of next bit, starting at 0
        self.pos
    }
}


#[cfg(test)]
mod test {
    use std::fs;
    use crate::parsing::parse;
    use super::*;

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

    #[test]
    fn test_huffman() {
        let data = fs::read("img/white_square.jpg").expect("Failed to read image");
        let tree = &mut parse(&data).huffman_tables[0].tree;

        assert_eq!(tree.forward(0), Ok(None));
        assert_eq!(tree.forward(0), Ok(Some(0)));
    
        assert_eq!(tree.forward(0), Ok(None));
        assert_eq!(tree.forward(1), Ok(None));
        assert_eq!(tree.forward(0), Ok(Some(1)));
    
        for _ in 0..8 {
            assert_eq!(tree.forward(1), Ok(None));
        }
        assert_eq!(tree.forward(0), Ok(Some(11)));

        for _ in 0..8 {
            assert_eq!(tree.forward(1), Ok(None));
        }
        assert_eq!(tree.forward(1), Err("No child"));
    }

    #[test]
    fn test_bit_stream() {
        let bytes = [0xFF, 0x00, 0b01001100];

        let mut bit_stream = BitStream::new(&bytes);
        
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
    }

    #[test]
    fn test_bit_stream_over() {
        let bytes = [0b11001100, 0xFF, 0x01, 0b11001100];

        let mut bit_stream = BitStream::new(&bytes);
        
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), None, "Next bit test");
        assert_eq!(bit_stream.next_bit(), None, "Next bit test");
    }

    #[test]
    fn test_bit_stream_get_coeff() {
        let bytes = [0b01110000, 0b10111110, 0b00001101];

        let mut bit_stream = BitStream::new(&bytes);
        
        assert_eq!(bit_stream.get_coeff(0), 0, "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.get_pos(), 0, "Next bit test");
        assert_eq!(bit_stream.get_coeff(3), 7, "Next bit test");
        assert_eq!(bit_stream.get_pos(), 3, "Next bit test");
        assert_eq!(bit_stream.get_coeff(3), -7, "Next bit test");
        assert_eq!(bit_stream.get_pos(), 6, "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(0), "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.get_coeff(5), -16, "Next bit test");
        assert_eq!(bit_stream.get_coeff(6), 32, "Next bit test");
        assert_eq!(bit_stream.next_bit(), Some(1), "Next bit test");
        assert_eq!(bit_stream.get_coeff(3), 5, "Next bit test");
    }
}

