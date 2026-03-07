use super::DesiredSize;

#[derive(Clone)]
pub enum LayoutItem {
    Widget { desired_width: DesiredSize, desired_height: DesiredSize },
    Spacer(DesiredSize),
}
