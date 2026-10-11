//! History graph and segmented bar widgets.
//!
//! The graph is drawn here rather than with ratatui's `Sparkline`, which rounds bars down (a small
//! non-zero sample stays blank) and scales to the largest sample of all its data (not only the
//! columns on screen).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;

use super::theme::gradient;

/// Bar heights in one row: `▁` (1/8) up to `█` (8/8).
const BAR_LEVELS: u64 = 8;
/// Code point before `▁`: the bar of level `n` is `BAR_BASE + n`.
const BAR_BASE: u32 = 0x2580;
/// Filled cell of a full bar.
const FULL: char = '█';
/// Half-filled cell of a three-level bar.
const HALF: char = '▄';

/// Bar of segments side by side over its whole area, left to right: each segment takes its share
/// of `total` in cells, in its color; the rest of the bar stays blank. Cell edges are rounded from
/// the running sum, so the segments always add up to the rounded sum of their values.
pub(super) struct SegmentBar {
  segments: Vec<(u64, Color)>,
  total: u64,
}

impl SegmentBar {
  pub(super) fn new(segments: Vec<(u64, Color)>, total: u64) -> Self {
    Self { segments, total }
  }
}

impl Widget for SegmentBar {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    if area.is_empty() || self.total == 0 {
      return;
    }

    // offsets stay within `area.width`, as the running sum is capped at the total
    let edge = |sum: u64| {
      let sum = u128::from(sum.min(self.total));
      ((sum * u128::from(area.width) + u128::from(self.total) / 2) / u128::from(self.total)) as u16
    };
    let mut sum = 0u64;
    for (value, color) in self.segments {
      let start = edge(sum);
      sum = sum.saturating_add(value);
      for x in area.left() + start..area.left() + edge(sum) {
        for y in area.top()..area.bottom() {
          buf[(x, y)].set_char(FULL).set_fg(color);
        }
      }
    }
  }
}

/// History graph for newest-first samples over its whole area, right-aligned (newest sample on
/// the right): one solid bar per column, growing from the bottom row up in eighths of a row
/// (`▁`…`█`; blank / `▄` / `█` with `three_levels`), each colored by its own value on the load
/// gradient, all in one color, or each in a color of its own. Zero values leave the column blank.
pub(super) struct Graph<'a> {
  data: &'a [u64],
  max: Option<u64>,
  color: Option<Color>,
  colors: Option<&'a [Color]>,
  three_levels: bool,
}

impl<'a> Graph<'a> {
  /// Graph scaled to its largest visible sample, each bar in its load color.
  pub(super) fn new(data: &'a [u64]) -> Self {
    Self { data, max: None, color: None, colors: None, three_levels: false }
  }

  /// Bars in three levels (blank, `▄`, `█`) instead of eighths, for Apple Terminal, which draws
  /// gaps between the eighth blocks.
  pub(super) fn three_levels(mut self, three_levels: bool) -> Self {
    self.three_levels = three_levels;
    self
  }

  /// Value drawn at full height.
  pub(super) fn max(mut self, max: u64) -> Self {
    self.max = Some(max);
    self
  }

  /// Draws every bar in `color` instead of its load color.
  pub(super) fn color(mut self, color: Color) -> Self {
    self.color = Some(color);
    self
  }

  /// Colors of the samples, newest first like the data, instead of their load colors; samples
  /// past its end keep their load color.
  pub(super) fn colors(mut self, colors: &'a [Color]) -> Self {
    self.colors = Some(colors);
    self
  }
}

impl Widget for Graph<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
      return;
    }

    let visible = &self.data[..self.data.len().min(usize::from(area.width))];
    let max = self.max.unwrap_or_else(|| visible.iter().copied().max().unwrap_or(0)).max(1);
    let levels = u64::from(area.height) * BAR_LEVELS;
    for (i, (x, &value)) in (area.left()..area.right()).rev().zip(visible).enumerate() {
      let mut level = bar_level(value, max, levels);
      let own = self.colors.and_then(|colors| colors.get(i).copied());
      let color = self.color.or(own).unwrap_or_else(|| gradient(value as f64 / max as f64));
      for y in (area.top()..area.bottom()).rev() {
        if level == 0 {
          break;
        }

        let eighths = level.min(BAR_LEVELS);
        let symbol = bar_symbol(eighths, self.three_levels);
        if symbol != ' ' {
          buf[(x, y)].set_char(symbol).set_fg(color);
        }
        level -= eighths;
      }
    }
  }
}

/// Glyph of a cell filled `eighths` of a row from the bottom: `▁`…`█`, or with three levels (as
/// the original macmon in Apple Terminal) blank up to 1/8, `▄` up to 6/8 and `█` above.
fn bar_symbol(eighths: u64, three_levels: bool) -> char {
  match (three_levels, eighths) {
    (_, 0) | (true, 1) => ' ',
    (true, 2..=6) => HALF,
    (true, _) => FULL,
    (false, _) => char::from_u32(BAR_BASE + eighths.min(BAR_LEVELS) as u32).unwrap_or(' '),
  }
}

/// Bar height of `value` in eighths of a row out of `levels`, rounded up: non-zero values get at
/// least `▁`.
fn bar_level(value: u64, max: u64, levels: u64) -> u64 {
  if value == 0 {
    return 0;
  }

  // in u128, so large values don't overflow
  let level = (u128::from(value) * u128::from(levels)).div_ceil(u128::from(max.max(1)));
  level.min(u128::from(levels)) as u64
}

#[cfg(test)]
mod tests {
  use ratatui::buffer::Buffer;
  use ratatui::layout::Rect;
  use ratatui::style::Color;
  use ratatui::widgets::Widget;

  use super::{Graph, SegmentBar};
  use crate::tui::theme::gradient;

  fn draw(widget: impl Widget, width: u16, height: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    widget.render(buf.area, &mut buf);
    buf
  }

  fn rows(buf: &Buffer) -> Vec<String> {
    let row = |y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
    (0..buf.area.height).map(row).collect()
  }

  #[test]
  fn graph_bars_grow_across_rows() {
    // three rows, newest on the right: 100 % full, 50 % one and a half rows, 30 % (7.2 eighths
    // rounded up to 8) one row, a tiny value one eighth at the bottom, zero blank
    let buf = draw(Graph::new(&[100, 50, 30, 1, 0]).max(100), 5, 3);
    assert_eq!(rows(&buf), ["█", "▄█", "▁███"].map(|r| format!("{r:>5}")));

    // every level of a two-row bar, bottom row full before the top row starts
    let levels: Vec<u64> = (1..=16).rev().collect();
    let buf = draw(Graph::new(&levels).max(16), 16, 2);
    assert_eq!(rows(&buf), ["        ▁▂▃▄▅▆▇█", "▁▂▃▄▅▆▇█████████"]);

    // without a maximum: scaled to the largest sample on screen, the older 100 doesn't fit
    assert_eq!(rows(&draw(Graph::new(&[10, 20, 100]), 2, 2)), ["█ ", "██"]);
  }

  #[test]
  fn graph_colors_each_column_or_all_in_one_color() {
    // the terminal's green / yellow / red, by each column's own value
    let buf = draw(Graph::new(&[90, 50, 10]).max(100), 3, 4);
    for (x, color) in [(0, Color::Green), (1, Color::Yellow), (2, Color::Red)] {
      for y in (0..4).filter(|&y| buf[(x, y)].symbol() != " ") {
        assert_eq!(buf[(x, y)].fg, color, "{x}, {y}");
      }
    }

    // power graphs: every bar in one color, the largest one too
    let low = gradient(0.0);
    let buf = draw(Graph::new(&[4000, 2000, 0, 200]).color(low), 4, 2);
    assert_eq!(rows(&buf), ["   █", "▁ ██"]);
    for (x, y) in [(0, 1), (2, 1), (3, 1), (3, 0)] {
      assert_eq!(buf[(x, y)].fg, low, "{x}, {y}");
    }
  }

  #[test]
  fn graph_takes_a_color_per_sample() {
    // newest first: the two newest in their own colors, the oldest past the colors in its load
    // color
    let colors = [Color::Blue, Color::Magenta];
    let buf = draw(Graph::new(&[100, 100, 100]).max(100).colors(&colors), 3, 1);
    let fg: Vec<Color> = (0..3).map(|x| buf[(x, 0)].fg).collect();
    assert_eq!(fg, [Color::Red, Color::Magenta, Color::Blue]);
  }

  #[test]
  fn segment_bar_splits_its_width_by_share() {
    let (a, b, c) = (Color::Magenta, Color::Blue, Color::Cyan);
    // 10 cells for a total of 100: 25 / 30 / 20 take 3 / 3 / 2 cells, rounded from the running sum
    // (2.5 → 3, 5.5 → 6, 7.5 → 8), the free rest blank, on every row
    let buf = draw(SegmentBar::new(vec![(25, a), (30, b), (20, c)], 100), 10, 2);
    assert_eq!(rows(&buf), vec!["████████  "; 2]);
    let fg: Vec<Color> = (0..8).map(|x| buf[(x, 1)].fg).collect();
    assert_eq!(fg, [a, a, a, b, b, b, c, c]);

    // segments past the total are cut at the end of the bar; no total draws nothing
    assert_eq!(rows(&draw(SegmentBar::new(vec![(80, a), (80, b)], 100), 5, 1)), ["█████"]);
    assert_eq!(rows(&draw(SegmentBar::new(vec![(80, a)], 0), 5, 1)), ["     "]);
  }

  #[test]
  fn graph_in_three_levels_for_apple_terminal() {
    // the bar set of the original macmon in Apple Terminal: 1/8 blank, 2/8–6/8 `▄`, 7/8–8/8 `█`,
    // full cells below the top one
    let levels: Vec<u64> = (1..=16).rev().collect();
    let buf = draw(Graph::new(&levels).max(16).three_levels(true), 16, 2);
    assert_eq!(rows(&buf), ["         ▄▄▄▄▄██", " ▄▄▄▄▄██████████"]);
  }
}
