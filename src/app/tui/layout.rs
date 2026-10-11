//! Screen layout: the metric boxes in the top half of the screen and the process list in the
//! rest.
//!
//! The metrics box holds a grid of three columns: CPU, GPU and RAM on top, CPU / GPU power and
//! network below, so each usage box sits over its power box. Unlike the original, the CPU is one
//! box (the clusters go on its bottom border), and the ANE power box gave way to the network box:
//! the ANE is idle most of the time, and its power still counts in the power summary. The boxes of
//! a row split its width evenly, the rows split the height (the top row gets the odd one). One
//! structure at every size, only the scale changes.
//!
//! A blank row follows each row of boxes and the metrics box, so the text on the bottom border of
//! one box never sits right on the top border of the next. On small screens the blank rows go
//! first: before the graphs get shorter than `SPACED_GRAPH_ROWS` or the process list hides.

use ratatui::layout::{Margin, Rect};

/// Share of the screen height for the metrics box over the process list, in percent.
const METRICS_HEIGHT_PCT: u32 = 50;
/// Lowest metrics box over the process list: two rows of boxes with one graph row each, plus the
/// borders.
const METRICS_MIN_HEIGHT: u16 = 8;
/// Fewest process rows worth showing; with less room the process list is auto-hidden.
const PROC_MIN_ROWS: u16 = 3;
/// Rows of the process box besides the process rows: borders and the table header.
const PROC_CHROME: u16 = 3;
/// Fewest graph rows of a metric box with the blank rows; with less room they are left out.
const SPACED_GRAPH_ROWS: u16 = 2;

/// One box of the metrics box: a title over a history graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Metric {
  Cpu,
  Gpu,
  Ram,
  CpuPower,
  GpuPower,
  Network,
}

/// Rectangles for one frame. Hidden parts are `None` / left out; every rect is non-empty.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct LayoutPlan {
  /// Metrics box, borders included.
  pub(super) top: Option<Rect>,
  /// Metric boxes inside the metrics box, borders included: the top row left to right, then the
  /// bottom row.
  pub(super) boxes: Vec<(Metric, Rect)>,
  /// Process box, borders included.
  pub(super) proc: Option<Rect>,
}

/// Boxes of the top row: CPU, GPU and RAM.
const TOP_ROW: [Metric; 3] = [Metric::Cpu, Metric::Gpu, Metric::Ram];

/// Boxes of the bottom row: CPU and GPU power, network.
const BOTTOM_ROW: [Metric; 3] = [Metric::CpuPower, Metric::GpuPower, Metric::Network];

/// Height of the metrics box over the process list on a screen `height` rows tall:
/// `METRICS_HEIGHT_PCT` of it, rounded, but at least `METRICS_MIN_HEIGHT`.
fn metrics_height(height: u16) -> u16 {
  // a share of `height`, so it fits in u16
  let share = ((u32::from(height) * METRICS_HEIGHT_PCT + 50) / 100) as u16;
  share.max(METRICS_MIN_HEIGHT).min(height)
}

/// `count` (at least one) boxes side by side splitting the width of `row` evenly; the odd cells
/// are spread between them. Boxes without cells are left out.
fn split_row(row: Rect, count: usize) -> impl Iterator<Item = Rect> {
  let (width, count) = (u64::from(row.width), count as u64);
  // offsets stay within `row.width`, so they fit in u16
  let edge = move |i: u64| (width * i / count) as u16;
  (0..count)
    .map(move |i| Rect { x: row.x + edge(i), width: edge(i + 1) - edge(i), ..row })
    .filter(|r| !r.is_empty())
}

/// Whether `area` has room for the process list under the metrics box: at least `PROC_MIN_ROWS`
/// process rows. The process list setting doesn't matter.
pub(super) fn procs_fit(area: Rect) -> bool {
  !area.is_empty() && area.height - metrics_height(area.height) >= PROC_MIN_ROWS + PROC_CHROME
}

/// Splits `area` into the metrics box and the process box below it.
///
/// With the process list on (`procs`), the metrics box takes `METRICS_HEIGHT_PCT` of the height
/// and the process box the rest, after a blank row when it still has `PROC_MIN_ROWS` process rows
/// then; left with fewer than `PROC_MIN_ROWS` process rows, the process list is auto-hidden.
/// Without it, the metrics box takes the whole screen.
pub(super) fn compute_layout(area: Rect, procs: bool) -> LayoutPlan {
  let mut plan = LayoutPlan::default();
  if area.is_empty() {
    return plan;
  }

  let mut top = area;
  if procs && procs_fit(area) {
    top.height = metrics_height(area.height);
    let rest = area.height - top.height;
    let gap = u16::from(rest > PROC_MIN_ROWS + PROC_CHROME);
    plan.proc = Some(Rect { y: top.bottom() + gap, height: rest - gap, ..area });
  }
  plan.top = Some(top);

  // a blank row under each row of boxes when both keep their graph rows
  let inner = top.inner(Margin::new(1, 1));
  let gap = u16::from(inner.height >= 2 * (SPACED_GRAPH_ROWS + 2) + 2);
  let boxes = inner.height - 2 * gap;
  let upper = boxes.div_ceil(2);
  let rows = [
    (Rect { height: upper, ..inner }, TOP_ROW),
    (Rect { y: inner.y + upper + gap, height: boxes - upper, ..inner }, BOTTOM_ROW),
  ];
  for (row, metrics) in rows {
    if row.is_empty() {
      continue;
    }
    plan.boxes.extend(metrics.into_iter().zip(split_row(row, metrics.len())));
  }

  plan
}

#[cfg(test)]
mod tests {
  use ratatui::layout::{Margin, Rect};

  use super::{PROC_CHROME, PROC_MIN_ROWS, compute_layout, procs_fit};

  #[test]
  fn metrics_take_half_and_procs_the_rest() {
    // (width, height, metrics box height): 50 % of the height, rounded, but at least two rows of
    // boxes with a graph row each
    for (width, height, top) in [(200, 50, 25), (110, 33, 17), (80, 24, 12), (100, 15, 8)] {
      let plan = compute_layout(Rect::new(0, 0, width, height), true);
      let ctx = format!("{width}x{height}");
      assert_eq!(plan.top, Some(Rect::new(0, 0, width, top)), "{ctx}");
      // after a blank row
      assert_eq!(plan.proc, Some(Rect::new(0, top + 1, width, height - top - 1)), "{ctx}");
    }
    // 100x14: no room for the blank row, the process list right under the metrics box
    let plan = compute_layout(Rect::new(0, 0, 100, 14), true);
    assert_eq!(plan.proc, Some(Rect::new(0, 8, 100, 6)));
  }

  #[test]
  fn hidden_or_auto_hidden_list_gives_metrics_the_full_height() {
    let area = Rect::new(0, 0, 120, 40);
    let plan = compute_layout(area, false);
    assert_eq!((plan.top, plan.proc), (Some(area), None));
    // 38 rows inside: a blank row under each row of boxes, 18 rows for each
    let rows: Vec<(u16, u16)> = plan.boxes.iter().map(|(_, r)| (r.y, r.height)).collect();
    assert_eq!(rows, [(1, 18), (1, 18), (1, 18), (20, 18), (20, 18), (20, 18)]);

    // rows of boxes 4 high (2 graph rows) with the blank rows, from 10 rows inside; with 9, none
    let rows = |height| {
      let plan = compute_layout(Rect::new(0, 0, 120, height), false);
      plan.boxes.iter().map(|(_, r)| (r.y, r.height)).step_by(3).collect::<Vec<_>>()
    };
    assert_eq!(rows(12), [(1, 4), (6, 4)]);
    assert_eq!(rows(11), [(1, 5), (6, 4)]);

    // 100x13: an 8 rows metrics box would leave 5 rows, one short of the smallest process box
    let small = Rect::new(0, 0, 100, 13);
    let plan = compute_layout(small, true);
    assert_eq!((plan.top, plan.proc), (Some(small), None));
    assert!(!procs_fit(small) && procs_fit(Rect { height: 14, ..small }));
  }

  #[test]
  fn boxes_stay_inside_and_never_overlap() {
    let widths = [1, 2, 3, 5, 10, 69, 80, 110, 200, 400];
    let heights = [1, 2, 3, 5, 8, 13, 14, 24, 50, 120];
    for (width, height) in widths.into_iter().flat_map(|w| heights.map(|h| (w, h))) {
      let area = Rect::new(3, 2, width, height);
      for procs in [false, true] {
        let plan = compute_layout(area, procs);
        let ctx = format!("{area:?} procs={procs}");

        // the metrics box on top, the process box under it (after a blank row at most) to the
        // bottom
        let top = plan.top.expect("metrics box");
        assert_eq!(plan.proc.is_some(), procs && procs_fit(area), "{ctx}");
        match plan.proc {
          Some(proc) => {
            let gap = proc.y - top.bottom();
            assert!(gap <= 1 && proc.bottom() == area.bottom(), "{ctx}: {proc:?}");
            assert_eq!((proc.x, proc.width), (area.x, area.width), "{ctx}");
            assert!(proc.height >= PROC_MIN_ROWS + PROC_CHROME, "{ctx}");
          }
          None => assert_eq!(top, area, "{ctx}: metrics take the full height"),
        }

        // boxes sit inside the borders of the metrics box and never overlap
        let inner = top.inner(Margin::new(1, 1));
        for (i, (_, a)) in plan.boxes.iter().enumerate() {
          assert!(!a.is_empty() && inner.intersection(*a) == *a, "{a:?} out of {inner:?}: {ctx}");
          for (_, b) in &plan.boxes[i + 1..] {
            assert!(!a.intersects(*b), "{a:?} overlaps {b:?} in {ctx}");
          }
        }
      }
    }
  }
}
