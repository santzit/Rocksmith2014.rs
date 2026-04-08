use std::io::{Read, Write};
use crate::binary_helpers::*;
use super::bend_value::BendValue;

#[derive(Debug, Clone)]
pub struct BendData32 {
    pub bend_values: [BendValue; 32],
    pub used_count: i32,
}

impl Default for BendData32 {
    fn default() -> Self {
        BendData32 {
            bend_values: std::array::from_fn(|_| BendValue::default()),
            used_count: 0,
        }
    }
}

impl SngRead for BendData32 {
    fn sng_read<R: Read>(r: &mut R) -> crate::Result<Self> {
        let mut bend_values: [BendValue; 32] = std::array::from_fn(|_| BendValue::default());
        for v in bend_values.iter_mut() {
            *v = BendValue::sng_read(r)?;
        }
        let used_count = read_i32(r)?;
        Ok(BendData32 { bend_values, used_count })
    }
}

impl SngWrite for BendData32 {
    fn sng_write<W: Write>(&self, w: &mut W) -> crate::Result<()> {
        for v in &self.bend_values {
            v.sng_write(w)?;
        }
        write_i32(w, self.used_count)?;
        Ok(())
    }
}
