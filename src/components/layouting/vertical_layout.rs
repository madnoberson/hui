use std::array;

use super::DesiredSize;
use crate::components::common::Bounds;

#[derive(Debug, Clone, PartialEq)]
pub enum VerticalLayoutItem {
    Widget { width: Option<f32>, height: DesiredSize },
    Spacer(DesiredSize),
}

impl VerticalLayoutItem {
    pub const fn height(&self) -> &DesiredSize {
        match self {
            VerticalLayoutItem::Widget { height, .. } => height,
            VerticalLayoutItem::Spacer(height) => height,
        }
    }
}

#[must_use]
pub fn fixed_vertical_layout<const N: usize>(
    position: [f32; 2],
    size: [f32; 2],
    items: &[VerticalLayoutItem; N],
) -> [Option<Bounds>; N] {
    let total_height = size[1];
    let mut used_height: f32 = 0.0;
    let mut resolved: [Option<f32>; N] = array::repeat(None);

    expand_fixed(items, &mut resolved, &mut used_height);
    expand_constrained(items, &mut resolved, &mut used_height, total_height);
    expand_greedy(items, &mut resolved, &mut used_height, total_height);

    make_bounds(items, &resolved, position, size)
}

fn expand_fixed<const N: usize>(
    items: &[VerticalLayoutItem; N],
    resolved: &mut [Option<f32>; N],
    used_height: &mut f32,
) {
    for (i, item) in items.iter().enumerate() {
        if let DesiredSize::Fixed(height) = item.height() {
            resolved[i] = Some(*height);
            *used_height += height;
        }
    }
}

fn expand_constrained<const N: usize>(
    items: &[VerticalLayoutItem; N],
    resolved: &mut [Option<f32>; N],
    used_height: &mut f32,
    total_height: f32,
) {
    for (i, item) in items.iter().enumerate() {
        if let DesiredSize::Constrained { min_value, desired_value } =
            item.height()
        {
            let available_height = total_height - *used_height;
            let height = available_height.clamp(*min_value, *desired_value);

            resolved[i] = Some(height);
            *used_height += height;
        }
    }
}

fn expand_greedy<const N: usize>(
    items: &[VerticalLayoutItem; N],
    resolved: &mut [Option<f32>; N],
    used_height: &mut f32,
    total_height: f32,
) {
    let mut widgets_total_height = 0.0f32;
    let mut widget_weight = 0u64;
    let mut spacer_weight = 0u64;

    for item in items {
        if let DesiredSize::Greedy { weight, .. } = item.height() {
            match item {
                VerticalLayoutItem::Widget { .. } => {
                    widget_weight += *weight as u64
                }
                _ => spacer_weight += *weight as u64,
            }
        }
    }

    let remaining_height = (total_height - *used_height).max(0.0);
    for (i, item) in items.iter().enumerate() {
        if let DesiredSize::Greedy { min_value, weight } = item.height() {
            let height = match item {
                VerticalLayoutItem::Widget { .. } if widget_weight > 0 => {
                    let height = min_value
                        + remaining_height
                            * (*weight as f32 / widget_weight as f32);
                    widgets_total_height += height;
                    height
                }
                _ => continue,
            };
            resolved[i] = Some(height);
        }
    }

    let space_for_spacers = (remaining_height - widgets_total_height).max(0.0);
    for (i, item) in items.iter().enumerate() {
        if let DesiredSize::Greedy { min_value, weight } = item.height() {
            if let VerticalLayoutItem::Spacer(_) = item {
                let height = if spacer_weight > 0 {
                    min_value
                        + space_for_spacers
                            * (*weight as f32 / spacer_weight as f32)
                } else {
                    *min_value
                };
                resolved[i] = Some(height);
            }
        }
    }
}

fn make_bounds<const N: usize>(
    items: &[VerticalLayoutItem; N],
    resolved: &[Option<f32>; N],
    position: [f32; 2],
    size: [f32; 2],
) -> [Option<Bounds>; N] {
    let mut bounds: [Option<Bounds>; N] = std::array::repeat(None);
    let mut cursor_y = position[1];
    let max_x = position[0] + size[0];
    let max_y = position[1] + size[1];

    for (i, item) in items.iter().enumerate() {
        let Some(height) = resolved[i] else { continue };
        let fits_height = cursor_y + height <= max_y + f32::EPSILON;

        if let VerticalLayoutItem::Widget { width, .. } = item {
            let widget_width = width.unwrap_or(size[0]);
            let fits_width =
                position[0] + widget_width <= max_x + f32::EPSILON;

            let clip_rect = if fits_height && fits_width {
                [0.0, 0.0, f32::MAX, f32::MAX]
            } else {
                [position[0], position[1], size[1], size[0]]
            };
            let item_bounds = Bounds {
                position: [position[0], cursor_y],
                size: [widget_width, height],
                clip_rect,
            };
            bounds[i] = Some(item_bounds);
        }

        if !fits_height {
            break;
        }

        cursor_y += height;
    }

    bounds
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{
        Bounds, DesiredSize, VerticalLayoutItem, fixed_vertical_layout,
    };

    const DEFAULT_CLIP_RECT: [f32; 4] = [0.0, 0.0, f32::MAX, f32::MAX];

    // Two fixed widgets with a fixed spacer between them.
    // Both widgets fit.
    const ITEMS_1: &[VerticalLayoutItem; 3] = &[
        VerticalLayoutItem::Widget {
            width:  Some(10.0),
            height: DesiredSize::Fixed(10.0),
        },
        VerticalLayoutItem::Spacer(DesiredSize::Fixed(30.0)),
        VerticalLayoutItem::Widget {
            width:  Some(20.0),
            height: DesiredSize::Fixed(50.0),
        },
    ];
    const BOUNDS_1: [Option<Bounds>; 3] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [10.0, 10.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        None,
        Some(Bounds {
            position:  [0.0, 40.0],
            size:      [20.0, 50.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // Two fixed widgets pushed apart by a greedy spacer that
    // consumes all remaining space.
    const ITEMS_2: &[VerticalLayoutItem; 3] = &[
        VerticalLayoutItem::Widget {
            width:  Some(10.0),
            height: DesiredSize::Fixed(10.0),
        },
        VerticalLayoutItem::Spacer(DesiredSize::Greedy {
            min_value: 0.0,
            weight:    1,
        }),
        VerticalLayoutItem::Widget {
            width:  Some(20.0),
            height: DesiredSize::Fixed(10.0),
        },
    ];
    const BOUNDS_2: [Option<Bounds>; 3] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [10.0, 10.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        None,
        Some(Bounds {
            position:  [0.0, 90.0],
            size:      [20.0, 10.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // A single greedy widget with no other items: it should
    // fill the entire layout height.
    const ITEMS_3: &[VerticalLayoutItem; 1] = &[VerticalLayoutItem::Widget {
        width:  None,
        height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
    }];
    const BOUNDS_3: [Option<Bounds>; 1] = [Some(Bounds {
        position:  [0.0, 0.0],
        size:      [100.0, 100.0],
        clip_rect: DEFAULT_CLIP_RECT,
    })];

    // Two greedy widgets with equal weights split the
    // available space evenly.
    const ITEMS_4: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
    ];
    const BOUNDS_4: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 50.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 50.0],
            size:      [100.0, 50.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // Two greedy widgets with weights 1 and 3 split the space
    // in a 1:3 ratio.
    const ITEMS_5: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 3 },
        },
    ];
    const BOUNDS_5: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 25.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 25.0],
            size:      [100.0, 75.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // Constrained widget gets its desired_value when enough space
    // is available.
    const ITEMS_6: &[VerticalLayoutItem; 1] = &[VerticalLayoutItem::Widget {
        width:  None,
        height: DesiredSize::Constrained {
            min_value:     10.0,
            desired_value: 40.0,
        },
    }];
    const BOUNDS_6: [Option<Bounds>; 1] = [Some(Bounds {
        position:  [0.0, 0.0],
        size:      [100.0, 40.0],
        clip_rect: DEFAULT_CLIP_RECT,
    })];

    // Constrained widget gets as much space as available, clamped
    // between min_value and desired_value. A fixed widget consumes
    // 70px first, leaving 30px - less than desired_value (40px),
    // but more than min_value (10px), so the constrained widget
    // receives exactly 30px.
    const ITEMS_7: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(70.0),
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Constrained {
                min_value:     10.0,
                desired_value: 40.0,
            },
        },
    ];
    const BOUNDS_7: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 70.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 70.0],
            size:      [100.0, 30.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // Fixed widget takes priority over greedy: fixed gets its full
    // size, greedy gets only what remains.
    const ITEMS_8: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(80.0),
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
    ];
    const BOUNDS_8: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 80.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 80.0],
            size:      [100.0, 20.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // A widget that partially overflows the layout boundary gets a
    // clip_rect equal to the layout bounds. All subsequent widgets
    // get None.
    const ITEMS_9: &[VerticalLayoutItem; 3] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(80.0),
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(40.0), // overflows: 80 + 40 = 120 > 100
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(10.0),
        },
    ];
    const BOUNDS_9: [Option<Bounds>; 3] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 80.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 80.0],
            size:      [100.0, 40.0],
            clip_rect: [0.0, 0.0, 100.0, 100.0], // clipped to layout bounds
        }),
        None,
    ];

    // Widget with an explicit width smaller than the layout width
    // starts at the layout's x position.
    const ITEMS_10: &[VerticalLayoutItem; 1] = &[VerticalLayoutItem::Widget {
        width:  Some(30.0),
        height: DesiredSize::Fixed(30.0),
    }];
    const BOUNDS_10: [Option<Bounds>; 1] = [Some(Bounds {
        position:  [0.0, 0.0],
        size:      [30.0, 30.0],
        clip_rect: DEFAULT_CLIP_RECT,
    })];

    // Greedy spacer does not compete with greedy widgets: widgets get
    // their share first, then the spacer gets whatever remains.
    const ITEMS_11: &[VerticalLayoutItem; 3] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
        VerticalLayoutItem::Spacer(DesiredSize::Greedy {
            min_value: 0.0,
            weight:    1,
        }),
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Greedy { min_value: 0.0, weight: 1 },
        },
    ];
    const BOUNDS_11: [Option<Bounds>; 3] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 50.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        None,
        Some(Bounds {
            position:  [0.0, 50.0],
            size:      [100.0, 50.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // Layout with a non-zero origin: all positions are offset by
    // the layout position.
    const ITEMS_12: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(20.0),
        },
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(30.0),
        },
    ];
    const BOUNDS_12: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [50.0, 50.0],
            size:      [100.0, 20.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [50.0, 70.0],
            size:      [100.0, 30.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
    ];

    // A widget whose explicit width exceeds the layout width gets
    // a clip_rect equal to the layout bounds, even though it
    // fits vertically.
    const ITEMS_13: &[VerticalLayoutItem; 1] = &[VerticalLayoutItem::Widget {
        width:  Some(150.0), // wider than layout (100px)
        height: DesiredSize::Fixed(30.0),
    }];
    const BOUNDS_13: [Option<Bounds>; 1] = [Some(Bounds {
        position:  [0.0, 0.0],
        size:      [150.0, 30.0],
        clip_rect: [0.0, 0.0, 100.0, 100.0], // clipped to layout bounds
    })];

    // Widget overflows both horizontally and vertically - clip_rect
    // set to layout bounds.
    const ITEMS_14: &[VerticalLayoutItem; 2] = &[
        VerticalLayoutItem::Widget {
            width:  None,
            height: DesiredSize::Fixed(80.0),
        },
        VerticalLayoutItem::Widget {
            width:  Some(150.0),              // wider than layout
            height: DesiredSize::Fixed(40.0), // also overflows vertically
        },
    ];
    const BOUNDS_14: [Option<Bounds>; 2] = [
        Some(Bounds {
            position:  [0.0, 0.0],
            size:      [100.0, 80.0],
            clip_rect: DEFAULT_CLIP_RECT,
        }),
        Some(Bounds {
            position:  [0.0, 80.0],
            size:      [150.0, 40.0],
            clip_rect: [0.0, 0.0, 100.0, 100.0],
        }),
    ];

    #[rstest]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_1, BOUNDS_1)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_2, BOUNDS_2)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_3, BOUNDS_3)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_4, BOUNDS_4)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_5, BOUNDS_5)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_6, BOUNDS_6)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_7, BOUNDS_7)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_8, BOUNDS_8)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_9, BOUNDS_9)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_10, BOUNDS_10)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_11, BOUNDS_11)]
    #[case([50.0, 50.0], [100.0, 100.0], ITEMS_12, BOUNDS_12)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_13, BOUNDS_13)]
    #[case([0.0, 0.0], [100.0, 100.0], ITEMS_14, BOUNDS_14)]
    fn test_fixed_vertical_layout<const N: usize>(
        #[case] position: [f32; 2],
        #[case] size: [f32; 2],
        #[case] items: &[VerticalLayoutItem; N],
        #[case] expected_bounds: [Option<Bounds>; N],
    ) {
        let bounds = fixed_vertical_layout(position, size, items);
        assert_eq!(bounds, expected_bounds);
    }
}
