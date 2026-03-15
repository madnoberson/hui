use bytemuck::Zeroable;
use slotmap::SlotMap;

use super::{Rectangle, RectangleId};

pub struct RectangleStore {
    slots: SlotMap<RectangleId, usize>,
    bytes: Vec<u8>,
}

impl RectangleStore {
    #[must_use]
    #[inline(always)]
    pub fn new() -> Self { Self { slots: SlotMap::new(), bytes: Vec::new() } }

    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.slots.is_empty() }

    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize { self.slots.len() }

    #[must_use]
    #[inline(always)]
    pub fn bytes(&mut self) -> &[u8] {
        &self.bytes[..self.slots.len() * Rectangle::SIZE]
    }

    #[must_use]
    pub fn add(&mut self, rect: &Rectangle) -> RectangleId {
        let rect_offset = self.bytes.len();
        let rect_bytes = bytemuck::bytes_of(rect);
        self.bytes.extend_from_slice(rect_bytes);
        self.slots.insert(rect_offset)
    }

    #[must_use]
    pub fn remove(&mut self, id: RectangleId) -> Option<Rectangle> {
        let mut removed_rect = Rectangle::zeroed();

        let removed_rect_offset = self.slots.remove(id)?;
        let last_rect_offset = self.bytes.len() - Rectangle::SIZE;

        let removed_rect_bytes =
            &self.bytes[last_rect_offset..last_rect_offset + Rectangle::SIZE];
        bytemuck::bytes_of_mut(&mut removed_rect)
            .copy_from_slice(removed_rect_bytes);

        if removed_rect_offset != last_rect_offset {
            self.bytes
                .copy_within(last_rect_offset.., removed_rect_offset);

            if let Some(moved_slot) =
                self.slots.values_mut().find(|o| **o == last_rect_offset)
            {
                *moved_slot = removed_rect_offset;
            }
        }
        self.bytes.truncate(self.bytes.len() - Rectangle::SIZE);

        Some(removed_rect)
    }

    #[must_use]
    #[inline(always)]
    pub fn get_mut(&mut self, id: RectangleId) -> Option<&mut Rectangle> {
        let rect_offset = *self.slots.get(id)?;

        let rect_bytes =
            &mut self.bytes[rect_offset..rect_offset + Rectangle::SIZE];
        let rect = bytemuck::from_bytes_mut(rect_bytes);

        Some(rect)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Rectangle, RectangleStore};

    #[rstest]
    fn test_rectangle_store() {
        let mut rectangle_store = RectangleStore::new();

        let rect = Rectangle::builder()
            .mvp([[1.0, 0.0, 0.0, 0.0]; 4])
            .fill_color([1.0, 0.0, 0.0, 1.0])
            .border_color([0.0, 0.0, 0.0, 1.0])
            .corner_radii([4.0, 4.0, 4.0, 4.0])
            .shadow_color([0.0, 0.0, 0.0, 0.5])
            .clip_rect([0.0, 0.0, 1920.0, 1080.0])
            .half_size([50.0, 25.0])
            .border_size(1.0)
            .shadow_spread(0.0)
            .shadow_offset([0.0, 0.0])
            .shadow_blur(0.0)
            .build();
        let rect_id = rectangle_store.add(&rect);

        let mut_ref_to_rect = rectangle_store.get_mut(rect_id).unwrap();
        let new_rect_fill_color = [0.0, 1.0, 0.0, 1.0];
        mut_ref_to_rect.fill_color = new_rect_fill_color;

        let removed_rect = rectangle_store.remove(rect_id).unwrap();
        assert_eq!(removed_rect.fill_color, new_rect_fill_color);
    }
}
