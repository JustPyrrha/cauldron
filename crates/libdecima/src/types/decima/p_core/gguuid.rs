use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
#[repr(C)]
pub struct GGUUID {
    pub data0: u8,
    pub data1: u8,
    pub data2: u8,
    pub data3: u8,
    pub data4: u8,
    pub data5: u8,
    pub data6: u8,
    pub data7: u8,
    pub data8: u8,
    pub data9: u8,
    pub data10: u8,
    pub data11: u8,
    pub data12: u8,
    pub data13: u8,
    pub data14: u8,
    pub data15: u8,
}

impl Display for GGUUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:x}{:x}{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}{:x}{:x}{:x}{:x}",
            self.data0,
            self.data1,
            self.data2,
            self.data3,
            self.data4,
            self.data5,
            self.data6,
            self.data7,
            self.data8,
            self.data9,
            self.data10,
            self.data11,
            self.data12,
            self.data13,
            self.data14,
            self.data15
        ))
    }
}

impl Debug for GGUUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("GGUUID {{ {} }}", self))
    }
}
