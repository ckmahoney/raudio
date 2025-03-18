use super::*;

/// mix of three different percs.
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::Mix(vec![
    (0.5, contour_stem(conf, melody, arf)),
    (0.5, contour_stem(conf, melody, arf)),
  ])
}
