use super::*;

/// mix of three different percs.
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  contour_stem(conf, melody, arf)
}
