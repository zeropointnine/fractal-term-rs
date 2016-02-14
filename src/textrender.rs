use ansi;
use matrix::Matrix;


pub struct TextRender {
    width: i32,
    height: i32,
    char_mapper: CharMapper,
}

impl TextRender {
    pub fn new(width: i32, height: i32, ceil: u16) -> TextRender {
        let cm = CharMapper::new(&" .,:;i*xo*O08%X#@".to_string(), ceil);
        let tr = TextRender {
            width: width,
            height: height,
            char_mapper: cm,
        };
        tr
    }

    pub fn set_charset(&mut self, charset: &String) {
        self.char_mapper.set_chars(charset);
    }
    pub fn set_ceil(&mut self, ceil: u16) {
        self.char_mapper.set_ceil(ceil);
    }

    pub fn clear(&self) {
        print!("{}", ansi::CLEAR);
    }

    pub fn render(&self, nummap: &mut Matrix<u16>) {

        if false {
            println!("{:?}\n", nummap);
            return;
        }

        let data = &mut nummap.vec();

        let data_width = data[0].len() as i32;
        let hspan = data_width / self.width;

        let data_height = data.len() as i32;
        let vspan = data_height / self.height;

        // TODO: println!("hspan {} vspan {}", hspan, vspan);

        // data's width must be a multiple of width, and data's height must be a multiple of height
        assert!(data_width % self.width == 0);
        assert!(data_height % self.height == 0);

        for row_num in 0..self.height {
            self.render_row(row_num, data);
        }
    }

    fn render_row(&self, row_num: i32, data: &Vec<Vec<u16>>) {

        // temp
        // let mut chars = vec!(0x21u8; self.width as usize);
        // let s = String::from_utf8(chars).unwrap();
        // print!("{}{}", ansi::move_cursor(row_num + 1, 1), s);


        let mut chars = vec!(' '; self.width as usize);  // TODO: use single instance

        let row = &data[row_num as usize];

        for col in 0..row.len() {
            let val = row[col];
            chars[col] = self.char_mapper.value_to_char(val);
        }

        // char vector to string:  // TODO: can this be optimized?
        let s = chars.iter().cloned().collect::<String>();
        print!("{}{}", ansi::move_cursor(row_num + 1, 1), s);
    }
}

struct CharMapper {
    chars: Vec<char>,
    ceil: u16,
    step: f64,
}

impl CharMapper {
    fn new(charset: &String, ceil: u16) -> CharMapper {
        let chars = charset.chars().collect();
        let step = ceil as f64 / charset.len() as f64;
        CharMapper {
            chars: chars,
            ceil: ceil,
            step: step,
        }
    }

    fn set_chars(&mut self, charset: &String) {
        self.chars = charset.chars().collect();
        self.step = self.chars.len() as f64 / self.ceil as f64;
    }

    fn set_ceil(&mut self, ceil: u16) {
        self.ceil = ceil;
        self.step = self.chars.len() as f64 / self.ceil as f64;
    }

    fn value_to_char(&self, value: u16) -> char {
        let mut i = (value as f64 / self.step) as usize;
        if i > self.chars.len() - 1 {
            i = self.chars.len() - 1;
        }
        self.chars[i]
    }
}
