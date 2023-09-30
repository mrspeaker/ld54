//! Genetic information.
use crate::prelude::*;

/// A gene name.
pub struct Gene<'a> {
    pub name: &'a str,
    pub sequence: &'a str,
}

/// A trait carried by an organism.
pub trait Stat {
    fn gene(&self) -> &'static Gene<'static>;
}

pub struct Chonk(f32);
impl Chonk {
    pub const GENE: Gene<'static> = gene!("chonkiness");
}

#[cfg(test)]
mod test {
    #[test]
    fn test_gene() {
        use crate::prelude::*;
        let size = gene!("size");
        assert_eq!(size.sequence, "tcgc");
    }
}
