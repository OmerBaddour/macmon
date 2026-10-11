//! Help overlay (`?`): the keys, the mouse, and what the less obvious values mean.

use Row::{Key, Section};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Clear;

use super::boxes::{Titles, cells, draw_box, hint};
use super::theme::{heading, text};

/// One line of the help.
enum Row {
  Section(&'static str),
  /// Keys (or what to do with the mouse) and what they do.
  Key(&'static str, &'static str),
}

const HELP: &[Row] = &[
  Section("Keys"),
  Key("q", "quit"),
  Key("p", "show / hide the processes"),
  Key("r", "CPU and GPU usage: scaled / active"),
  Key("- / +", "update interval"),
  Key("[ / ]", "process list interval"),
  Key("/", "filter the processes"),
  Key("s / S", "sort by the next column / reverse"),
  Key("↑ ↓", "select a process"),
  Key("Esc", "clear the selection and the filter"),
  Key("?", "show / hide this help"),
  Section("Mouse"),
  Key("click", "a column header to sort, a row to select"),
  Section("Notes"),
  Key("CPU%", "100% is one fully busy core"),
  Key("CPU box", "the whole CPU; its clusters below, fastest first"),
  Key("scaled", "usage weighted by frequency; active: share of time busy"),
  Key("POWER -", "not available for other users' processes"),
  Key("DOWN / UP", "network per second (46.8M = 46.8 MB/s); no loopback"),
  Key("RAM bar", "wired / app / compressed / cached; free blank"),
  Key("RAM graph", "colored by memory pressure, as Activity Monitor"),
  Key("SWAP ⇅", "swapping now: yellow, red from 10 MB/s"),
  Key("DISK", "reads (R) over writes (W) on one scale; no disk images"),
  Key("NET", "download over upload on one scale; VPN traffic counts once"),
];

/// Cells of the key column, the gap after it included.
const KEYS_WIDTH: usize = 14;

/// The help text: section names bold, keys bold, the rest plain.
fn lines() -> Vec<Line<'static>> {
  HELP
    .iter()
    .map(|line| match *line {
      Section(name) => Line::from(heading(name)),
      Key(keys, what) => Line::from(vec![heading(format!("  {keys:KEYS_WIDTH$}")), text(what)]),
    })
    .collect()
}

/// Draws the help in a box centered over `area`, from line `scroll` on (in a window too short for
/// all of it, ↑ / ↓ scroll it), clamped so the box is never left half empty. Returns the clamped
/// scroll.
pub(super) fn render(f: &mut Frame, area: Rect, scroll: usize) -> usize {
  let lines = lines();
  // the text with a blank column on both sides, and the borders
  let width = lines.iter().map(Line::width).max().unwrap_or(0) + 4;
  let height = lines.len() + 2;
  let width = cells(width).min(area.width);
  let height = cells(height).min(area.height);
  let x = area.x + (area.width - width) / 2;
  let y = area.y + (area.height - height) / 2;
  let rect = Rect::new(x, y, width, height);
  if rect.is_empty() {
    return scroll; // nothing to draw on (a window resized to zero rows or columns)
  }

  let shown = usize::from(height.saturating_sub(2));
  let scroll = scroll.min(lines.len().saturating_sub(shown));
  let titles = Titles::new(heading("help")).right(hint("Esc", "close"));

  f.render_widget(Clear, rect);
  let inner = draw_box(f, rect, titles);
  let buf = f.buffer_mut();
  let text_width = inner.width.saturating_sub(2);
  for (line, y) in lines.iter().skip(scroll).take(shown).zip(inner.y..) {
    buf.set_line(inner.x + 1, y, line, text_width);
  }
  scroll
}
