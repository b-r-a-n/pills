use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Urgency {
    pub amount: f32,
}

const MIN_DROP_PERIOD: f32 = 0.35;

pub(crate) fn apply(
    augments: Query<(&Urgency, &InBoard), Added<InBoard>>,
    mut boards: Query<&mut BoardConfig>,
) {
    for (augment, board_id) in &augments {
        if let Ok(mut config) = boards.get_mut(**board_id) {
            if config.drop_period > augment.amount {
                if config.drop_period - augment.amount > MIN_DROP_PERIOD {
                    config.drop_period -= augment.amount;
                } else {
                    config.drop_period = MIN_DROP_PERIOD;
                }
            }
        }
    }
}
