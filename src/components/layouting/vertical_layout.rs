use std::mem::MaybeUninit;

use super::{Bounds, DesiredSize, LayoutItem};

pub fn vertical_layout<const N: usize>(
    position: [f32; 2],
    size: [f32; 2],
    items: &[LayoutItem; N],
) -> [Bounds; N] {
    let [available_width, available_height] = size;

    // let (final_width, min_height, desired_heights) =
    //     initial_computing::compute(available_width, available_height, items);
    let mut bounds: [MaybeUninit<Bounds>; N] =
        std::array::from_fn(|_| MaybeUninit::uninit());

    unsafe { MaybeUninit::array_assume_init(bounds) }
}

// mod initial_computing {
//     use std::mem::MaybeUninit;

//     use arrayvec::ArrayVec;

//     use super::{DesiredSize, LayoutItem};

//     #[must_use]
//     #[inline(always)]
//     pub fn compute<const N: usize>(
//         available_width: f32,
//         available_height: f32,
//         items: &[LayoutItem; N],
//     ) -> (f32, f32, ArrayVec<DesiredSize, N>) {
//         let (mut final_width, mut min_height) = (0.0_f32, 0.0_f32);

//         let mut desired_heights: [MaybeUninit<DesiredSize>; N] =
//             std::array::from_fn(|_| MaybeUninit::uninit());

//         for (i, item) in items.iter().enumerate() {
//             match item {
//                 LayoutItem::Widget { desired_width, desired_height } => {
//                     let no_more_space_left = take_widget_into_account(
//                         desired_width,
//                         desired_height,
//                         &mut final_width,
//                         &mut min_height,
//                         available_width,
//                         available_height,
//                     );
//                     if no_more_space_left {
//                         break;
//                     }
//                     desired_heights[i].write(desired_height.clone());
//                 }
//                 LayoutItem::Spacer(desired_height) => {
//                     desired_heights[i].write(desired_height.clone());
//                 }
//             }
//         }

//         // SAFETY: All N elements were initialized above in the
//         // loop.
//         let desired_heights =
//             unsafe { MaybeUninit::array_assume_init(desired_heights) };
//         (final_width, min_height, desired_heights)
//     }

//     #[must_use]
//     #[inline(always)]
//     fn take_widget_into_account(
//         desired_width: &DesiredSize,
//         desired_height: &DesiredSize,
//         final_width: &mut f32,
//         min_height: &mut f32,
//         available_width: f32,
//         available_height: f32,
//     ) -> bool {
//         true
//     }
// }

// #[cfg(test)]
// mod tests {
//     use rstest::rstest;

//     use super::*;

//     const POSITION_1: [f32; 2] = [0.0, 0.0];
//     const SIZE_1: [f32; 2] = [100.0, 100.0];

//     const LAYOUT_ITEMS_1: &'static [LayoutItem] = &[
//         LayoutItem::Widget {
//             desired_width:  DesiredSize::Fixed(100.0),
//             desired_height: DesiredSize::Fixed(50.0),
//         },
//         LayoutItem::Widget {
//             desired_width:  DesiredSize::Fixed(100.0),
//             desired_height: DesiredSize::Fixed(50.0),
//         },
//     ];
//     const BOUNDS_1: &'static [Bounds] = &[
//         Bounds { position: [0.0, 0.0], size: [100.0, 50.0] },
//         Bounds { position: [0.0, 50.0], size: [100.0, 50.0] },
//     ];

//     #[rstest]
//     #[case(POSITION_1, SIZE_1, LAYOUT_ITEMS_1, BOUNDS_1)]
//     fn test_vertical_layout(
//         #[case] position: [f32; 2],
//         #[case] size: [f32; 2],
//         #[case] items: &[LayoutItem],
//         #[case] expected_bounds: &[Bounds],
//     ) {
//         // let mut bounds = make_zeroed_bounds(expected_bounds.len());
//         // vertical_layout(position, size, items, &mut bounds);
//         // assert_eq!(bounds.as_slice(), expected_bounds);
//     }

//     fn make_zeroed_bounds(bounds_len: usize) -> Vec<Bounds> {
//         vec![Bounds { position: [0.0, 0.0], size: [0.0, 0.0] }; bounds_len]
//     }
// }
