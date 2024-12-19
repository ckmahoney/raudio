use super::*;

/// mix of three different percs.
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  simple_stem(conf, melody, arf)
}
